#![no_main] // Required, we build this with `-bundle`.
use objc2::{ClassType, MainThreadOnly, define_class};
use objc2_foundation::NSString;
use objc2_foundation::{
    NSSearchPathDirectory, NSSearchPathDomainMask, NSSearchPathForDirectoriesInDomains,
    NSDictionary, ns_string,
};
use objc2_xc_test::XCTestCase;
use objc2_xc_ui_automation::{
    XCUIApplication, XCUIDevice, XCUIElementTypeQueryProvider, XCUIScreenshot,
    XCUIScreenshotProviding,
};

define_class!(
    #[unsafe(super = XCTestCase)]
    #[thread_kind = MainThreadOnly]
    struct TestCase;

    impl TestCase {
        #[unsafe(method(setUp))]
        fn set_up(&self) {
            // Test setup code in here.
        }

        #[unsafe(method(tearDown))]
        fn tear_down(&self) {
            // Test teardown code in here.
        }

        #[unsafe(method(testSimple))]
        fn test_simple(&self) {
            let app = XCUIApplication::new(self.mtm());

            // SIMCTL_CHILD_DINGHY_LLVM_PROFILE_FILE
            if let Ok(val) = std::env::var("DINGHY_LLVM_PROFILE_FILE") {
                let envs: objc2::rc::Retained<NSDictionary<NSString, NSString>> =
                    NSDictionary::from_slices(
                        &[
                        ns_string!("LLVM_PROFILE_FILE"),
                        ns_string!("LLVM_PROFILE_VERBOSE_ERRORS"),
                        ],
                        &[&NSString::from_str(val.as_str()), ns_string!("1")],
                    );

                app.setLaunchEnvironment(&envs);
            }

            app.launch();
            let text_view = app.textFields().elementBoundByIndex(0);
            text_view.tap();
            text_view.typeText(&NSString::from_str(" THIS TEXT IS FROM XCTEST"));
            save_screenshot(&app.screenshot());

            let device = XCUIDevice::sharedDevice(self.mtm());
            let siri = device.siriService();
            siri.activateWithVoiceRecognitionText(&NSString::from_str("What is the capital of germany?"));

            std::thread::sleep(std::time::Duration::from_millis(500));

            device.pressButton(objc2_xc_ui_automation::XCUIDeviceButton::Home);
            device.pressButton(objc2_xc_ui_automation::XCUIDeviceButton::Home);

        }
    }
);

/// Load and initialize the class such that XCTest can see it.
#[ctor::ctor]
unsafe fn setup() {
    let _ = TestCase::class();
}

fn save_screenshot(screenshot: &XCUIScreenshot) {
    let path = NSSearchPathForDirectoriesInDomains(
        NSSearchPathDirectory::DocumentDirectory,
        NSSearchPathDomainMask::UserDomainMask,
        true,
    );
    if let Some(path) = path.firstObject() {
        let path = path.to_string();
        let path = std::path::Path::new(&path).join(format!("screenshot.png"));
        let res = screenshot
            .PNGRepresentation()
            .writeToFile_atomically(&NSString::from_str(path.to_str().unwrap()), false);
        assert!(res, "failed writing screenshot");
    }
}
