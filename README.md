# sendevent

Send Linux input events from the output of [getevent](https://source.android.com/devices/input/getevent).

getevent source code:

https://cs.android.com/android/platform/superproject/+/master:system/core/toolbox/getevent.c

## Usage

```
sendevent

USAGE:
    sendevent [OPTIONS]

OPTIONS:
        --device <DEVICE>
    -h, --help               Print help information
        --path <PATH>
```

`--device <DEVICE>` is required if the device field is omitted from the input.

If `--path <PATH>` is omitted, it will read events from stdin.

Capture events and save them to `/data/local/tmp/output`:

```
adb shell getevent -l -t -q > /data/local/tmp/output
```

The verbose mask must be cleared (via `-q` or `-v0`) or set to 1 (via `-v1`), otherwise additional device information will be printed.

Do not use the flag `-d` as well.

Do not use the flag `-n` because there will be no newlines between events in the output.

Older versions of getevent does not disable buffering on stdout so the above command may not work well. Use `adb exec-out` to fix the problem.

Save the events to `output` on the adb host computer:

```
adb exec-out getevent -l -t -q > output
```

Push the output file to the Android device:

```
adb push output /data/local/tmp/output
```

After capturing the events and building the executable, push the executable to the Android device:

```
adb push <sendevent> /data/local/tmp/
```

`<sendevent>` is the path to the built executable.

Replay events:

```
adb shell /data/local/tmp/sendevent --path /data/local/tmp/output
```

## Build

Download NDK from https://developer.android.com/ndk/downloads.

Problem with NDK version r23:

https://github.com/rust-lang/rust/pull/85806

Get the ABI using adb:

```
adb shell getprop ro.product.cpu.abi
```

Determine the triple corresponding to the ABI:

| ABI         | triple                |
| ----------- | --------------------- |
| armeabi-v7a | arm-linux-androideabi |
| aarch64-v8a | aarch64-linux-android |
| x86         | i686-linux-android    |
| x86_64      | x86_64-linux-android  |

https://android.googlesource.com/platform/ndk/+/master/docs/BuildSystemMaintainers.md#architectures

Build by specifying the linker with an environment variable:

```
CARGO_TARGET_<triple>_LINKER=<linker> cargo build --target <triple>
```

`<linker>` is `<ndk>/toolchains/llvm/prebuilt/<host_tag>/bin/<triple><api>-clang`.

`<ndk>` is the path to the downloaded NDK. `<api>` is the minSdkVersion. `<host_tag>` has to match the downloaded NDK:

| NDK OS Variant | Host Tag       |
| -------------- | -------------- |
| macOS          | darwin-x86_64  |
| Linux          | linux-x86_64   |
| 32-bit Windows | windows        |
| 64-bit Windows | windows-x86_64 |

https://developer.android.com/ndk/guides/other_build_systems#overview

More information on specifying linkers:

https://doc.rust-lang.org/cargo/reference/config.html#targettriplelinker

Example to build ABI aarch64-v8a with android-ndk-r22b and minSdkVersion 24 on Linux:

```
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=android-ndk-r22b/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang cargo build --target aarch64-linux-android
```

## Implementation Details

If there are timestamps in the input, the program will delay writing the event until a time relative to the first event time.

Actually only timestamps of events with event type `EV_SYN` and event code `SYN_REPORT` matter.

From https://docs.kernel.org/input/event-codes.html:

> Used to synchronize and separate events into packets of input data changes occurring at the same moment in time. For example, motion of a mouse may set the REL_X and REL_Y values for one motion, then emit a SYN_REPORT. The next motion will emit more REL_X and REL_Y values and send another SYN_REPORT.

If you look at the output of getevent, you will see that the events between two `SYN_REPORT` events share the same timestamp.

Therefore, there is no need to delay writing events other than `SYN_REPORT` events. The program will immediately write all events except events with event type `EV_SYN` and event code `SYN_REPORT`.

## Note

From https://docs.kernel.org/input/event-codes.html:

> The input protocol is a stateful protocol. Events are emitted only when values of event codes have changed.

Therefore, getevent only report events when the state changes. The starting state when running getevent is unknown and it may be different from the state when running sendevent. This may not be desired.
