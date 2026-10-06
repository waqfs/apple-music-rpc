use objc2::{
    msg_send,
    rc::Retained,
    runtime::{AnyClass, AnyObject},
};
use objc2_foundation::NSString;

pub struct AppleMusicBridge {
    app: Retained<AnyObject>,
}

impl AppleMusicBridge {
    pub fn new() -> Result<Self, String> {
        let clazz = AnyClass::get(c"SBApplication")
            .ok_or_else(|| "error: SBApplication class is not available".to_string())?;

        let bundle_id = NSString::from_str("com.apple.Music");
        let app: Option<Retained<AnyObject>> =
            unsafe { msg_send![clazz, applicationWithBundleIdentifier: &*bundle_id] };
        let app = app.ok_or_else(|| "error: failed to bridge the Apple Music app".to_string())?;
        Ok(Self { app })
    }
}
