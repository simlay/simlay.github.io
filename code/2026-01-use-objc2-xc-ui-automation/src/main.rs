use objc2::{
    define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::AnyObject,
    ClassType, Ivars, MainThreadMarker, MainThreadOnly,
};

use objc2_foundation::{NSDictionary, NSObject, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    UIApplication, UIApplicationDelegate, UIApplicationLaunchOptionsKey, UIScreen, UITextField,
    UIViewController, UIWindow,
};

define_class!(
    // SAFETY:
    // - `NSObject` does not have any subclassing requirements.
    // - `AppDelegate` does not implement `Drop`.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct AppDelegate {
        window: std::cell::RefCell<Option<Retained<UIWindow>>>,
    }

    impl AppDelegate {
        // Called by `UIApplication::main`.
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(Ivars::<Self> {
                window: std::cell::RefCell::new(None),
            });
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl UIApplicationDelegate for AppDelegate {
        #[unsafe(method(application:didFinishLaunchingWithOptions:))]
        unsafe fn did_finish_launching_with_options(
            &self,
            application: &UIApplication,
            launch_options: Option<&NSDictionary<UIApplicationLaunchOptionsKey, AnyObject>>,
        ) -> bool {
            let mtm = MainThreadMarker::new().expect("Failed to get mtm");
            let window = UIWindow::new(mtm);
            window.setFrame(UIScreen::mainScreen(mtm).bounds());
            window.makeKeyAndVisible();
            *self.window().borrow_mut() = Some(window);

            let view_controller = UIViewController::new(mtm);
            let text_field = UITextField::new(mtm);
            text_field.setText(Some(&NSString::from_str("THIS IS THE DEFAULT TEXT")));

            self.window().borrow().as_ref().unwrap().setRootViewController(Some(&view_controller));
            view_controller.setView(Some(&text_field));
            text_field.setBackgroundColor(Some(&objc2_ui_kit::UIColor::whiteColor()));
            true
        }
    }
);

fn main() {
    set_llvm_profile_write();
    println!("Hello, World!");
    let mtm = MainThreadMarker::new().unwrap();
    let delegate_class = NSString::from_class(AppDelegate::class());
    UIApplication::main(None, Some(&delegate_class), mtm);
}


fn set_llvm_profile_write() {
    if std::env::var("LLVM_PROFILE_FILE").is_ok() {
        let _will_terminate_observer = create_observer(
            &objc2_foundation::NSNotificationCenter::defaultCenter(),
            unsafe { objc2_ui_kit::UIApplicationDidEnterBackgroundNotification },
            move |_notification| {
                unsafe extern "C" {
                    safe fn __llvm_profile_write_file() -> std::ffi::c_int;
                }
                let res = __llvm_profile_write_file();
                assert_eq!(res, 0);
            },
        );
    }
}

fn create_observer(
    center: &objc2_foundation::NSNotificationCenter,
    name: &objc2_foundation::NSNotificationName,
    handler: impl Fn(&objc2_foundation::NSNotification) + 'static,
) -> objc2::rc::Retained<objc2::runtime::ProtocolObject<dyn objc2_foundation::NSObjectProtocol>> {
    let block = block2::RcBlock::new(
        move |notification: std::ptr::NonNull<objc2_foundation::NSNotification>| {
            handler(unsafe { notification.as_ref() });
        },
    );

    unsafe { center.addObserverForName_object_queue_usingBlock(Some(name), None, None, &block) }
}
