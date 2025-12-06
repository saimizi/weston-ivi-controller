// Controller module - Core IVI surface management

pub mod events;
pub mod ivi_wrapper;
pub mod notifications;
pub mod state;
pub mod validation;

pub use events::{EventContext, EventListeners};
pub use ivi_wrapper::{IviLayer, IviLayoutApi, IviSurface};
pub use notifications::{Notification, NotificationData, NotificationManager, NotificationType};
pub use state::StateManager;
pub use validation::{
    validate_opacity, validate_orientation, validate_position, validate_size, validate_z_order,
    ValidationError,
};
