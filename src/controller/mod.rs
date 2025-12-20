// Controller module - Core IVI surface management

pub mod events;
pub mod id_assignment;
pub mod notifications;
pub mod state;
pub mod subscriptions;
pub mod validation;

pub use events::{EventContext, EventListeners};
pub use id_assignment::{
    IdAssignmentConfig, IdAssignmentError, IdAssignmentInfo, IdAssignmentManager,
    IdAssignmentResult, IdAssignmentStats,
};
pub use notifications::{Notification, NotificationData, NotificationManager, NotificationType};
pub use state::StateManager;
pub use subscriptions::SubscriptionManager;
pub use validation::{
    validate_opacity, validate_orientation, validate_position, validate_size, validate_z_order,
    ValidationError,
};
