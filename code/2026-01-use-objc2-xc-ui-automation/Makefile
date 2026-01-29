
EMULATOR='iPhone 16e'
WRAPPER_ID=com.simlay.net.Dinghy

build:
	cargo build --target aarch64-apple-ios-sim --all --all-targets

bundle: build
	cp ./target/aarch64-apple-ios-sim/debug/use-objc2-xc-ui-automation ./RustApp.app/

install: bundle
	xcrun simctl install $(EMULATOR) ./RustApp.app/

run: install
	xcrun simctl launch --console --terminate-running-process $(EMULATOR) $(WRAPPER_ID)

record:
	xcrun simctl io $(EMULATOR) recordVideo -f record.mp4

watch:
	cargo watch -s 'make run' -w ./src -w ./Cargo.toml -w ./ui_tests/

compress-recording:
	ffmpeg -i record.mp4 -c:v libx264 -crf 30 record-compressed.mp4

ui-tests-build:
	cargo build -p ui_tests

ui-tests-bundle-tools: ui-tests-build
	cp $(shell xcode-select --print-path)/Platforms/IPhoneSimulator.platform/Developer/Library/Xcode/Agents/XCTRunner.app/XCTRunner                ./RustUITests.app/
	cp -r $(shell xcode-select --print-path)/Platforms/IPhoneSimulator.platform/Developer/Library/Frameworks                                       ./RustUITests.app/
	cp -r $(shell xcode-select --print-path)/Platforms/IPhoneSimulator.platform/Developer/Library/PrivateFrameworks/XCTestCore.framework           ./RustUITests.app/Frameworks/
	cp -r $(shell xcode-select --print-path)/Platforms/IPhoneSimulator.platform/Developer/Library/PrivateFrameworks/XCTestSupport.framework        ./RustUITests.app/Frameworks/
	cp -r $(shell xcode-select --print-path)/Platforms/IPhoneSimulator.platform/Developer/Library/PrivateFrameworks/XCUnit.framework               ./RustUITests.app/Frameworks/
	cp -r $(shell xcode-select --print-path)/Platforms/IPhoneSimulator.platform/Developer/Library/PrivateFrameworks/XCTAutomationSupport.framework ./RustUITests.app/Frameworks/

ui-tests-bundle: ui-tests-bundle-tools
	cp ./target/aarch64-apple-ios-sim/debug/ui_tests  ./RustUITests.app/Plugins/DinghyUITests.xctest/

ui-tests-install: ui-tests-bundle
	xcrun simctl install $(EMULATOR) RustUITests.app/

XCWRAPPER_ID=com.simlay.net.DinghyUITests.xctrunner

GET_CONTAINER=xcrun simctl get_app_container $(EMULATOR)
DINGHY_CONTAINER_CMD=$(GET_CONTAINER) $(WRAPPER_ID)
DINGHY_CONTAINER=$(shell $(DINGHY_CONTAINER_CMD) data)

XCTEST_CONTAINER_CMD=$(GET_CONTAINER) $(XCWRAPPER_ID)
XCTEST_CONTAINER=$(shell $(XCTEST_CONTAINER_CMD) data)

ui-tests-xctest-configuration: ui-tests-install
	cat ui_tests/ui_tests.xctestconfiguration.base | \
		sed "s:UI_TEST_WRAPPER_CONTAINER:$(shell $(XCTEST_CONTAINER_CMD)):g" | \
		sed "s:WRAPPER_APP_CONTAINER:$(shell $(DINGHY_CONTAINER_CMD)):g" | \
		sed "s:WRAPPER_APP_ID:$(WRAPPER_ID):g" \
		> ui_tests/ui_tests.xctestconfiguration

LAUNCH=xcrun simctl launch --console --terminate-running-process $(EMULATOR)

ui-tests-run: install ui-tests-install ui-tests-xctest-configuration
	SIMCTL_CHILD_DINGHY_LLVM_PROFILE_FILE="$(DINGHY_CONTAINER)/Documents/dinghy.profraw" SIMCTL_CHILD_LLVM_PROFILE_FILE="$(XCTEST_CONTAINER)/Documents/xctrunner.profraw" $(LAUNCH) $(XCWRAPPER_ID) 2>&1 | tee $(PWD)/stdout.txt
	make ui-tests-cp-screenshot

ui-tests-cp-screenshot:
	cp "$(XCTEST_CONTAINER)/Documents/screenshot.png" ui_tests.png
	sips -Z 640 ui_tests.png

LLVM_COV=$(shell rustc --print sysroot)/lib/rustlib/aarch64-apple-darwin/bin/llvm-cov
LLVM_PROFDATA=$(shell rustc --print sysroot)/lib/rustlib/aarch64-apple-darwin/bin/llvm-profdata

COV_IGNORE_LIST=--ignore-filename-regex='/.cargo/registry' --ignore-filename-regex='/.rustup' --ignore-filename-regex='/objc2'

COV_REPORT=$(LLVM_COV) report -Xdemangler=rustfilt --use-color   $(COV_IGNORE_LIST) -instr-profile
COV_EXPORT=$(LLVM_COV) export -Xdemangler=rustfilt --format lcov $(COV_IGNORE_LIST) -instr-profile

ui-tests-cov: ui-tests-run
	mkdir -p target/cov
	cp "$(XCTEST_CONTAINER)/Documents/xctrunner.profraw" ./target/cov/xctrunner.profraw
	cp "$(DINGHY_CONTAINER)/Documents/dinghy.profraw" ./target/cov/dinghy.profraw
	du -hs ./target/cov/*.profraw
	$(LLVM_PROFDATA)  merge -sparse ./target/cov/xctrunner.profraw -o ./target/cov/xctrunner.profdata
	$(LLVM_PROFDATA)  merge -sparse    ./target/cov/dinghy.profraw -o ./target/cov/dinghy.profdata
	$(COV_REPORT) ./target/cov/xctrunner.profdata ./target/aarch64-apple-ios-sim/debug/ui_tests
	$(COV_REPORT) ./target/cov/dinghy.profdata    ./target/aarch64-apple-ios-sim/debug/use-objc2-xc-ui-automation
	$(COV_EXPORT) ./target/cov/xctrunner.profdata ./target/aarch64-apple-ios-sim/debug/ui_tests        > ./target/cov/xctrunner-lcov.info
	$(COV_EXPORT) ./target/cov/dinghy.profdata    ./target/aarch64-apple-ios-sim/debug/use-objc2-xc-ui-automation > ./target/cov/dinghy-lcov.info
	genhtml ./target/cov/dinghy-lcov.info ./target/cov/xctrunner-lcov.info -o ./target/cov/

.EXPORT_ALL_VARIABLES:
SIMCTL_CHILD_RUST_BACKTRACE=full
SIMCTL_CHILD_RUST_LOG=trace
SIMCTL_CHILD_XCTestConfigurationFilePath=$(PWD)/ui_tests/ui_tests.xctestconfiguration
SIMCTL_CHILD_LLVM_PROFILE_VERBOSE_ERRORS=1
