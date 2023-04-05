#!/bin/sh
set -o pipefail

EXECUTABLE=$1
ARGS=${@:2}

IDENTIFIER="com.simlay.net.RustRunner"
DISPLAY_NAME="RustRunner"
BUNDLE_NAME=${DISPLAY_NAME}.App
EXECUTABLE_NAME=$(basename ${EXECUTABLE})
BUNDLE_PATH=$(dirname $EXECUTABLE)/${BUNDLE_NAME}

PLIST="<?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <!DOCTYPE plist PUBLIC \"-//Apple Computer//DTD PLIST 1.0//EN\"
    \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
    <plist version=\"1.0\">
    <dict>
    <key>CFBundleIdentifier</key>
    <string>${IDENTIFIER}</string>
    <key>CFBundleDisplayName</key>
    <string>${DISPLAY_NAME}</string>
    <key>CFBundleName</key>
    <string>${BUNDLE_NAME}</string>
    <key>CFBundleExecutable</key>
    <string>${EXECUTABLE_NAME}</string>
    <key>CFBundleVersion</key>
    <string>0.0.1</string>
    <key>CFBundleShortVersionString</key>
    <string>0.0.1</string>
    <key>CFBundleDevelopmentRegion</key>
    <string>en_US</string>
    <key>UILaunchStoryboardName</key>
    <string></string>
    <key>CFBundleIconFiles</key>
    <array>
    </array>
    <key>LSRequiresIPhoneOS</key>
    <true/>
    </dict>
</plist>"

mkdir -p ${BUNDLE_PATH}
rm ${BUNDLE_PATH}/* 2> /dev/null

echo $PLIST > ${BUNDLE_PATH}/Info.plist
cp ${EXECUTABLE} ${BUNDLE_PATH}

# Some simctl helper functions
ios_runtime() {
    xcrun simctl list -j runtimes ios | \
        jq -r '.[] | sort_by(.identifier)[0].identifier'
}
ios_devices() {
    xcrun simctl list -j devices ios | \
        jq ".devices.\"$(ios_runtime)\"" | jq 'sort_by(.deviceTypeIdentifier)'
}

ios_devices_booted() {
    ios_devices | jq '[.[] | select(.state == "Booted")]'
}

ios_devices_shutdown() {
    ios_devices | jq '[.[] | select(.state == "Shutdown")]'
}

ios_devices_get_id() {
    booted=$(ios_devices_booted)
    if [ "$booted" != "[]" ]; then
        echo $booted | jq -r '.[0].udid'
    else
        device_id=$(ios_devices_shutdown | jq -r '.[0].udid')
        xcrun simctl boot $device_id
        echo $device_id
    fi
}
DEVICE_ID=$(ios_devices_get_id)

# Install/reinstall the app, Start the app in the iOS simulator but wait for
# the debugger to attach
xcrun simctl uninstall ${DEVICE_ID} ${IDENTIFIER}
xcrun simctl install ${DEVICE_ID} ${BUNDLE_PATH}
INSTALLED_PATH=$(xcrun simctl get_app_container ${DEVICE_ID} ${IDENTIFIER})
APP_STDOUT=${INSTALLED_PATH}/stdout
APP_STDERR=${INSTALLED_PATH}/stderr

APP_PID=$(\
    xcrun simctl launch -w \
    --stdout=${APP_STDOUT} \
    --stderr=${APP_STDERR} \
    --terminate-running-process ${DEVICE_ID} ${IDENTIFIER} ${ARGS} \
    | awk -F: '{print $2}')

tail -f ${APP_STDOUT} &
tail -f ${APP_STDERR} >&2 &

# Attach to the app using lldb.
LLDB_SCRIPT_FILE=$(mktemp /tmp/lldb_script.XXXXX)
echo "attach ${APP_PID}" >> ${LLDB_SCRIPT_FILE}
echo "continue" >> ${LLDB_SCRIPT_FILE}
echo "quit" >> ${LLDB_SCRIPT_FILE}

LLDB_OUT_FILE=$(mktemp /tmp/lldb_script.XXXXX)
lldb -s ${LLDB_SCRIPT_FILE} > ${LLDB_OUT_FILE}

# This is the stdout LLDB:
#
# (lldb) command source -s 0 '/tmp/lldb.xijJI'
# Executing commands in '/tmp/lldb.xijJI'.
# (lldb) attach  89772
# Process 89772 stopped
# * thread #1, stop reason = signal SIGSTOP
#     frame #0: 0x0000000102b00a40 dyld`_dyld_start
# dyld`:
# ->  0x102b00a40 <+0>:  mov    x0, sp
#     0x102b00a44 <+4>:  and    sp, x0, #0xfffffffffffffff0
#     0x102b00a48 <+8>:  mov    x29, #0x0
#     0x102b00a4c <+12>: mov    x30, #0x0
# Target 0: (test_runner-8652616abdef98d9) stopped.
# Executable module set to "THE PATH TO THE IOS BUNDLED APP IN THE APP".
# Architecture set to: arm64e-apple-ios-simulator.
# (lldb) continue
# Process 89772 resuming
# Process 89772 exited with status = 0 (0x00000000)
# (lldb) quit


# Parse the output of lldb to retrieve the status code.
STATUS_CODE=$(\
    cat ${LLDB_OUT_FILE} | \
    grep "Process \d\+ exited with status = \d\+" | \
    grep ${APP_PID} | \
    grep -o "= \d\+" | \
    sed 's/= //g'\
)
exit ${STATUS_CODE}
