//! EdgeBot ROS2 Bridge
//!
//! Provides ROS2 integration using the rclrs crate. Enables publishing and
//! subscribing to ROS2 topics with zero-copy message passing where possible.
//! Integrates with edgebot-core for AI inference on sensor data.

use rclrs::{Context, Node, Publisher, Subscription, Qos, RclRsError};

/// ROS2 Bridge for EdgeBot AI.
///
/// Handles ROS2 node lifecycle and provides methods to create publishers
/// and subscribers. Designed to work with edgebot-core's inference engine.
pub struct Ros2Bridge {
    _context: Context,
    node: Node,
}

impl Ros2Bridge {
    /// Create a new ROS2 bridge with a named node.
    ///
    /// # Arguments
    /// * `node_name` - Name of the ROS2 node
    /// * `namespace` - Namespace for the node (e.g., "/robot1")
    ///
    /// # Returns
    /// A `Ros2Bridge` instance or an error if ROS2 context/node creation fails.
    pub fn new(node_name: &str, namespace: &str) -> Result<Self, RclRsError> {
        let mut context = Context::new(None)?;
        let node = Node::new(&mut context, node_name, namespace)?;
        Ok(Self { _context: context, node })
    }

    /// Get a reference to the underlying ROS2 node.
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Create a publisher on a topic.
    ///
    /// # Arguments
    /// * `topic` - Topic name (e.g., "/detections")
    /// * `qos` - Quality of Service profile (use `Qos::default()` or customize)
    ///
    /// Returns a `Publisher` for the given message type `M`.
    pub fn create_publisher<M: rclrs::MessageType>(
        &self,
        topic: &str,
        qos: Qos,
    ) -> Publisher<M> {
        self.node.create_publisher(topic, qos)
    }

    /// Create a subscription on a topic with a callback.
    ///
    /// The callback is invoked when a message arrives. It receives a reference
    /// to the message, which can be used directly for zero-copy processing.
    ///
    /// # Arguments
    /// * `topic` - Topic name (e.g., "/camera/image_raw")
    /// * `qos` - Quality of Service profile
    /// * `callback` - Function to process incoming messages; signature `Fn(&M)`
    ///
    /// Returns a `Subscription` that can be used to manage the subscription.
    pub fn create_subscription<M, F>(
        &self,
        topic: &str,
        qos: Qos,
        callback: F,
    ) -> Subscription<M>
    where
        M: rclrs::MessageType + 'static,
        F: Fn(&M) + Send + Sync + 'static,
    {
        self.node.create_subscription(topic, qos, callback)
    }
}
