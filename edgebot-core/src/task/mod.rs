//! Model task abstraction for edge AI tasks.
//!
//! This module provides a trait `ModelTask` for defining executable tasks
//! such as object detection and pathfinding. It includes example implementations
//! for YOLOv8 object detection and A* pathfinding.

use burn::backend::Backend;
use burn::tensor::Tensor;
use crate::inference::{InferenceEngine, InferenceError};
use thiserror::Error;

/// Errors that can occur during task execution.
#[derive(Error, Debug)]
pub enum TaskError {
    /// Invalid input provided to the task.
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// Inference error from the underlying model.
    #[error("inference error: {0}")]
    Inference(#[from] InferenceError),
    /// Pathfinding specific error.
    #[error("pathfinding error: {0}")]
    Pathfinding(String),
    /// Task-specific error (e.g., decoding failure).
    #[error("task error: {0}")]
    Task(String),
}

/// Trait for tasks that can be executed on edge devices.
///
/// A task encapsulates a specific AI or computational workload with configurable
/// runtime parameters. The trait is not generic over backend; implementations
/// may use any backend internally.
pub trait ModelTask {
    /// The input type for this task.
    type Input;
    /// The output type for this task.
    type Output;
    
    /// Execute the task with the given input.
    fn run(&self, input: Self::Input) -> Result<Self::Output, TaskError>;
    
    /// Get the batch size for this task.
    fn batch_size(&self) -> usize;
    
    /// Set the batch size for this task.
    fn set_batch_size(&mut self, batch_size: usize);
}

// ============================================================================
// YOLOv8 Object Detection Implementation
// ============================================================================

/// Configuration for YOLOv8 inference.
pub struct YoloConfig {
    /// Confidence threshold for detections (0.0 - 1.0).
    pub confidence_threshold: f64,
    /// IoU threshold for non-maximum suppression (0.0 - 1.0).
    pub iou_threshold: f64,
    /// Number of classes in the model.
    pub num_classes: usize,
}

/// Result of YOLOv8 object detection.
pub struct YoloOutput {
    /// Detections for each batch element.
    /// Vector of vectors: outer vec is batch, inner vec is detections in that batch.
    pub detections: Vec<Vec<Detection>>,
}

/// A single object detection.
pub struct Detection {
    /// Bounding box in normalized coordinates (0-1).
    pub bbox: BBox,
    /// Class identifier.
    pub class_id: usize,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f64,
}

/// Bounding box with normalized coordinates.
pub struct BBox {
    /// Center x coordinate (normalized 0-1).
    pub x: f64,
    /// Center y coordinate (normalized 0-1).
    pub y: f64,
    /// Width (normalized 0-1).
    pub width: f64,
    /// Height (normalized 0-1).
    pub height: f64,
}

/// YOLOv8 object detection task.
///
/// Generic over Burn backend `B`. Uses an `InferenceEngine` to run the model.
pub struct YoloV8<B: Backend> {
    engine: InferenceEngine<B>,
    batch_size: usize,
    config: YoloConfig,
}

impl<B: Backend> YoloV8<B> {
    /// Create a new YOLOv8 task from an inference engine and configuration.
    pub fn new(engine: InferenceEngine<B>, config: YoloConfig) -> Self {
        Self {
            engine,
            batch_size: 1, // default
            config,
        }
    }
    
    /// Get the confidence threshold.
    pub fn confidence_threshold(&self) -> f64 {
        self.config.confidence_threshold
    }
    
    /// Set the confidence threshold.
    pub fn set_confidence_threshold(&mut self, threshold: f64) {
        self.config.confidence_threshold = threshold.max(0.0).min(1.0);
    }
    
    /// Get the IoU threshold.
    pub fn iou_threshold(&self) -> f64 {
        self.config.iou_threshold
    }
    
    /// Set the IoU threshold.
    pub fn set_iou_threshold(&mut self, threshold: f64) {
        self.config.iou_threshold = threshold.max(0.0).min(1.0);
    }
}

impl<B: Backend> ModelTask for YoloV8<B> {
    type Input = Tensor<B>;
    type Output = YoloOutput;
    
    fn run(&self, input: Tensor<B>) -> Result<Self::Output, TaskError> {
        // Run inference
        let output = self.engine.forward(input)?;
        
        // Decode YOLO output to detections
        let detections = decode_yolo_output(output, &self.config)?;
        
        Ok(YoloOutput { detections })
    }
    
    fn batch_size(&self) -> usize {
        self.batch_size
    }
    
    fn set_batch_size(&mut self, batch_size: usize) {
        self.batch_size = batch_size.max(1);
    }
}

/// Decode raw YOLO model output into detection results.
///
/// # Arguments
/// * `output` - Raw model output tensor of shape [batch, 4+num_classes, num_boxes]
/// * `config` - YOLO configuration with thresholds
///
/// Returns a vector of detection vectors per batch element.
fn decode_yolo_output<B: Backend>(
    output: Tensor<B>,
    config: &YoloConfig,
) -> Result<Vec<Vec<Detection>>, TaskError> {
    // Convert tensor to CPU vector
    let data = output.into_data().into_vec();
    let dims = output.dims();
    
    // Expected shape: [batch, 4+num_classes, num_boxes]
    if dims.len() != 3 {
        return Err(TaskError::InvalidInput(format!(
            "Expected 3D tensor, got {}D",
            dims.len()
        )));
    }
    
    let batch_size = dims[0];
    let feature_dim = dims[1];
    let num_boxes = dims[2];
    
    // We expect feature_dim == 4 + num_classes
    if feature_dim != 4 + config.num_classes {
        return Err(TaskError::InvalidInput(format!(
            "Expected feature dimension {} (4+num_classes), got {}",
            4 + config.num_classes, feature_dim
        )));
    }
    
    let mut all_detections = Vec::with_capacity(batch_size);
    
    for b in 0..batch_size {
        let mut batch_detections = Vec::new();
        
        for box_idx in 0..num_boxes {
            // Compute offset in the flat array
            // Tensor layout: [batch][feature][box] (row-major contiguous)
            // Index = b*(feature_dim*num_boxes) + f*(num_boxes) + box_idx
            let offset = |f| b * (feature_dim * num_boxes) + f * num_boxes + box_idx;
            
            let cx = data[offset(0)] as f64;
            let cy = data[offset(1)] as f64;
            let w = data[offset(2)] as f64;
            let h = data[offset(3)] as f64;
            
            // Get class probabilities (starting at index 4)
            let mut max_class_prob = 0.0;
            let mut class_id = 0;
            for class in 0..config.num_classes {
                let prob = data[offset(4 + class)] as f64;
                if prob > max_class_prob {
                    max_class_prob = prob;
                    class_id = class;
                }
            }
            
            // YOLOv8 confidence is the max class probability
            let confidence = max_class_prob;
            
            if confidence >= config.confidence_threshold {
                let bbox = BBox {
                    x: cx,
                    y: cy,
                    width: w,
                    height: h,
                };
                batch_detections.push(Detection {
                    bbox,
                    class_id,
                    confidence,
                });
            }
        }
        
        all_detections.push(batch_detections);
    }
    
    Ok(all_detections)
}

// ============================================================================
// A* Pathfinding Implementation
// ============================================================================

/// A 2D grid for pathfinding.
pub struct Grid {
    /// Width of the grid (number of columns)
    pub width: usize,
    /// Height of the grid (number of rows)
    pub height: usize,
    /// Cell walkability: true = walkable, false = obstacle.
    pub cells: Vec<Vec<bool>>,
}

/// Query for pathfinding: find path from start to goal on a grid.
pub struct PathfindingQuery {
    /// The grid to navigate.
    pub grid: Grid,
    /// Start position (row, column).
    pub start: (usize, usize),
    /// Goal position (row, column).
    pub goal: (usize, usize),
}

/// Result of a pathfinding operation.
pub struct Path {
    /// Sequence of grid coordinates from start to goal inclusive.
    pub path: Vec<(usize, usize)>,
    /// Total cost of the path (number of steps).
    pub cost: usize,
}

/// A* pathfinding algorithm.
///
/// Implements the classic A* algorithm with Manhattan distance heuristic.
/// Supports batch processing for multiple queries (batch size > 1) by
/// processing them sequentially (the batch_size is mainly for API compatibility).
pub struct AStar {
    batch_size: usize,
    // optional heuristic weight (for Weighted A*)
    heuristic_weight: f64,
}

impl AStar {
    /// Create a new A* pathfinder.
    pub fn new() -> Self {
        Self {
            batch_size: 1,
            heuristic_weight: 1.0,
        }
    }
    
    /// Set the heuristic weight (1.0 for classic A*).
    pub fn set_heuristic_weight(&mut self, weight: f64) {
        self.heuristic_weight = weight.max(0.0);
    }
    
    /// Get the heuristic weight.
    pub fn heuristic_weight(&self) -> f64 {
        self.heuristic_weight
    }
}

impl Default for AStar {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelTask for AStar {
    type Input = PathfindingQuery;
    type Output = Path;
    
    fn run(&self, input: PathfindingQuery) -> Result<Self::Output, TaskError> {
        // Validate start and goal are within bounds and walkable
        let (sr, sc) = input.start;
        let (gr, gc) = input.goal;
        let grid = &input.grid;
        
        if sr >= grid.height || sc >= grid.width {
            return Err(TaskError::InvalidInput(format!(
                "Start position ({}, {}) out of bounds ({}x{})",
                sr, sc, grid.height, grid.width
            )));
        }
        if gr >= grid.height || gc >= grid.width {
            return Err(TaskError::InvalidInput(format!(
                "Goal position ({}, {}) out of bounds ({}x{})",
                gr, gc, grid.height, grid.width
            )));
        }
        
        if !grid.cells[sr][sc] {
            return Err(TaskError::InvalidInput("Start cell is not walkable".to_string()));
        }
        if !grid.cells[gr][gc] {
            return Err(TaskError::InvalidInput("Goal cell is not walkable".to_string()));
        }
        
        // Run A*
        let path = astar_search(&input)?;
        let cost = path.len().saturating_sub(1); // number of edges
        
        Ok(Path { path, cost })
    }
    
    fn batch_size(&self) -> usize {
        self.batch_size
    }
    
    fn set_batch_size(&mut self, batch_size: usize) {
        self.batch_size = batch_size.max(1);
    }
}

/// Perform A* search on a grid.
fn astar_search(query: &PathfindingQuery) -> Result<Vec<(usize, usize)>, TaskError> {
    use std::collections::{BinaryHeap, HashMap};
    use std::cmp::Ordering;
    
    let grid = &query.grid;
    let start = query.start;
    let goal = query.goal;
    
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum NodeState {
        Open,
        Closed,
    }
    
    // For ordering by f_score (f = g + h)
    #[derive(PartialEq, Eq)]
    struct Node {
        f: usize,
        g: usize,
        pos: (usize, usize),
    }
    
    impl Ord for Node {
        fn cmp(&self, other: &Self) -> Ordering {
            other.f.cmp(&self.f).then_with(|| self.g.cmp(&other.g))
        }
    }
    
    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    
    // g_score: cost from start to node
    let mut g_score: HashMap<(usize, usize), usize> = HashMap::new();
    // parent: for reconstructing path
    let mut parent: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    // state of nodes
    let mut state: HashMap<(usize, usize), NodeState> = HashMap::new();
    
    let mut open_set = BinaryHeap::new();
    
    // Initialize start node
    g_score.insert(start, 0);
    let h_start = heuristic(start, goal);
    open_set.push(Node {
        f: h_start,
        g: 0,
        pos: start,
    });
    state.insert(start, NodeState::Open);
    
    let directions = [(0,1), (1,0), (0,-1), (-1,0)]; // 4-connected
    
    while let Some(current) = open_set.pop() {
        if current.pos == goal {
            // Reconstruct path
            let mut path = Vec::new();
            let mut curr = goal;
            while curr != start {
                path.push(curr);
                curr = parent[&curr];
            }
            path.push(start);
            path.reverse();
            return Ok(path);
        }
        
        state.insert(current.pos, NodeState::Closed);
        
        let (r, c) = current.pos;
        for (dr, dc) in directions.iter() {
            let nr = r as isize + dr;
            let nc = c as isize + dc;
            if nr < 0 || nc < 0 {
                continue;
            }
            let nr = nr as usize;
            let nc = nc as usize;
            if nr >= grid.height || nc >= grid.width {
                continue;
            }
            if !grid.cells[nr][nc] {
                continue; // obstacle
            }
            let neighbor = (nr, nc);
            
            if let Some(NodeState::Closed) = state.get(&neighbor) {
                continue;
            }
            
            let tentative_g = g_score[&current.pos] + 1; // cost = 1 per move
            
            let neighbor_g = g_score.get(&neighbor).copied().unwrap_or(usize::MAX);
            if tentative_g < neighbor_g {
                // This path is better
                parent.insert(neighbor, current.pos);
                g_score.insert(neighbor, tentative_g);
                let h = heuristic(neighbor, goal);
                let f = tentative_g + h;
                open_set.push(Node {
                    f,
                    g: tentative_g,
                    pos: neighbor,
                });
                state.insert(neighbor, NodeState::Open);
            }
        }
    }
    
    Err(TaskError::Pathfinding("No path found".to_string()))
}

/// Manhattan distance heuristic.
fn heuristic(a: (usize, usize), b: (usize, usize)) -> usize {
    let (ar, ac) = a;
    let (br, bc) = b;
    ar.abs_diff(br) + ac.abs_diff(bc)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::wgpu::WgpuBackend;
    use burn::tensor::Tensor;
    use burn::tensor::Data;
    
    #[test]
    fn test_yolo_set_batch_size() {
        let device = WgpuBackend::Device::default();
        // Create a dummy linear model as placeholder (not a real YOLO)
        let linear = burn::nn::Linear::new(
            burn::nn::LinearConfig::new(3, 2)
        );
        let model = burn::nn::Model::new(linear);
        let engine = InferenceEngine::new(model, device);
        let config = YoloConfig {
            confidence_threshold: 0.5,
            iou_threshold: 0.4,
            num_classes: 80,
        };
        let mut yolo = YoloV8::new(engine, config);
        assert_eq!(yolo.batch_size(), 1);
        yolo.set_batch_size(4);
        assert_eq!(yolo.batch_size(), 4);
    }
    
    #[test]
    fn test_yolo_threshold_clamping() {
        let device = WgpuBackend::Device::default();
        let linear = burn::nn::Linear::new(burn::nn::LinearConfig::new(3, 2));
        let model = burn::nn::Model::new(linear);
        let engine = InferenceEngine::new(model, device);
        let config = YoloConfig {
            confidence_threshold: 0.5,
            iou_threshold: 0.4,
            num_classes: 80,
        };
        let mut yolo = YoloV8::new(engine, config);
        yolo.set_confidence_threshold(1.5);
        assert!(yolo.confidence_threshold() <= 1.0);
        yolo.set_confidence_threshold(-0.5);
        assert!(yolo.confidence_threshold() >= 0.0);
    }
    
    #[test]
    fn test_decode_yolo_output_simple() {
        // Create a simple output tensor manually using WgpuBackend
        let device = WgpuBackend::Device::default();
        // Tensor shape: [1, 6, 2] (batch=1, 2 classes, 2 boxes)
        let data = vec![
            // f0 (x): box0, box1
            0.3f32, 0.7,
            // f1 (y):
            0.4, 0.8,
            // f2 (w):
            0.1, 0.1,
            // f3 (h):
            0.2, 0.2,
            // f4 (class0):
            0.9, 0.2,
            // f5 (class1):
            0.1, 0.8,
        ];
        let data_tensor = Tensor::from_data(Data::from(data)).reshape([1, 6, 2]);
        
        let config = YoloConfig {
            confidence_threshold: 0.5,
            iou_threshold: 0.4,
            num_classes: 2,
        };
        
        let result = decode_yolo_output(data_tensor, &config).unwrap();
        
        assert_eq!(result.len(), 1); // batch size 1
        let detections = &result[0];
        // Should have 2 detections because both boxes have confidence >= 0.5
        assert_eq!(detections.len(), 2);
        
        // Check first detection (class0)
        let d0 = &detections[0];
        assert_eq!(d0.class_id, 0);
        assert!((d0.bbox.x - 0.3).abs() < 1e-5);
        assert!((d0.bbox.y - 0.4).abs() < 1e-5);
        assert!((d0.bbox.width - 0.1).abs() < 1e-5);
        assert!((d0.bbox.height - 0.2).abs() < 1e-5);
        assert!((d0.confidence - 0.9).abs() < 1e-5);
        
        // Check second detection (class1)
        let d1 = &detections[1];
        assert_eq!(d1.class_id, 1);
        assert!((d1.bbox.x - 0.7).abs() < 1e-5);
        assert!((d1.bbox.y - 0.8).abs() < 1e-5);
        assert!((d1.bbox.width - 0.1).abs() < 1e-5);
        assert!((d1.bbox.height - 0.2).abs() < 1e-5);
        assert!((d1.confidence - 0.8).abs() < 1e-5);
    }
    
    #[test]
    fn test_decode_yolo_output_threshold_filtering() {
        let device = WgpuBackend::Device::default();
        // Same as above but with one box having confidence below threshold
        let data = vec![
            0.3, 0.7,
            0.4, 0.8,
            0.1, 0.1,
            0.2, 0.2,
            0.9, 0.2,
            0.1, 0.8,
        ];
        let data_tensor = Tensor::from_data(Data::from(data)).reshape([1, 6, 2]);
        
        let config = YoloConfig {
            confidence_threshold: 0.95, // higher than 0.9
            iou_threshold: 0.4,
            num_classes: 2,
        };
        
        let result = decode_yolo_output(data_tensor, &config).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 0); // no detections pass threshold
    }
    
    #[test]
    fn test_astar_simple_path() {
        let grid = Grid {
            width: 3,
            height: 3,
            cells: vec![
                vec![true, true, true],
                vec![true, true, true],
                vec![true, true, true],
            ],
        };
        let query = PathfindingQuery {
            grid,
            start: (0, 0),
            goal: (2, 2),
        };
        let astar = AStar::new();
        let result = astar.run(query).unwrap();
        
        // Path should be length 5 (start + 4 steps = 5 nodes) for 2x2 Manhattan distance
        assert_eq!(result.path.len(), 5);
        assert_eq!(result.path[0], (0,0));
        assert_eq!(result.path[4], (2,2));
        assert_eq!(result.cost, 4);
    }
    
    #[test]
    fn test_astar_with_obstacle() {
        let grid = Grid {
            width: 3,
            height: 3,
            cells: vec![
                vec![true, false, true],
                vec![true, true, true],
                vec![true, false, true],
            ],
        };
        let query = PathfindingQuery {
            grid,
            start: (0, 0),
            goal: (0, 2),
        };
        let astar = AStar::new();
        let result = astar.run(query).unwrap();
        
        // The path should go down then right then up
        assert!(result.path.len() > 0);
        assert_eq!(result.path[0], (0,0));
        assert_eq!(result.path[result.path.len()-1], (0,2));
    }
    
    #[test]
    fn test_astar_unreachable() {
        let grid = Grid {
            width: 3,
            height: 3,
            cells: vec![
                vec![true, false, false],
                vec![false, false, false],
                vec![false, false, true],
            ],
        };
        let query = PathfindingQuery {
            grid,
            start: (0, 0),
            goal: (2, 2),
        };
        let astar = AStar::new();
        let result = astar.run(query);
        assert!(matches!(result, Err(TaskError::Pathfinding(_))));
    }
    
    #[test]
    fn test_astar_invalid_start_goal() {
        let grid = Grid {
            width: 2,
            height: 2,
            cells: vec![
                vec![true, true],
                vec![true, true],
            ],
        };
        let query = PathfindingQuery {
            grid,
            start: (2, 0), // out of bounds
            goal: (0, 0),
        };
        let astar = AStar::new();
        let result = astar.run(query);
        assert!(matches!(result, Err(TaskError::InvalidInput(_))));
        
        let query2 = PathfindingQuery {
            grid,
            start: (0, 0),
            goal: (0, 0),
        };
        let result2 = astar.run(query2);
        assert!(result2.is_ok()); // start=goal should be fine, returns path of length 1
        assert_eq!(result2.unwrap().path.len(), 1);
    }
}
