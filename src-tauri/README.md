# Tauri + Vite

Using Vite + React frontend at [/src-react](/src-react). Backend is decoupled from Tauri (frontend agnostic) at [/src-core](/src-core).

## Development

### Desktop (Windows, macOS, Linux)

Run the development server:
```bash
pnpm dev:desktop
```

Build for production:
```bash
pnpm build:desktop
```

### Android Setup

#### Prerequisites

1. **Java Development Kit (JDK 17 or later)** - Download from [oracle.com](https://www.oracle.com/java/technologies/downloads/) or use a package manager
2. **Android SDK** - Install via Android Studio or command line tools
3. **NDK (Native Development Kit)** - Required for Rust compilation
4. **Rust** - Install from [rustup.rs](https://rustup.rs/) with Android targets:
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
   ```
5. **Rust Analyzer** for IDE support (optional but recommended)

#### Environment Variables

Set these environment variables:
```bash
# Windows (PowerShell)
$env:JAVA_HOME = "C:\Program Files\Java\jdk-17" # Adjust path as needed
$env:ANDROID_HOME = "$env:USERPROFILE\AppData\Local\Android\Sdk"
$env:NDK_HOME = "$env:ANDROID_HOME\ndk\25.2.9519653" # Adjust NDK version as needed

# macOS/Linux
export JAVA_HOME=/path/to/jdk-17
export ANDROID_HOME=$HOME/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653
```

#### Initial Setup

Initialize the Android project:
```bash
pnpm init:android
```

#### Development & Testing

Start development server for Android:
```bash
pnpm dev:android
```

Build debug APK:
```bash
pnpm build:android
```

Install APK on connected device:
```bash
pnpm install:android
```

Ensure you have a device connected via USB with USB debugging enabled, or an Android emulator running.

#### Signing Release Builds

Android APKs must be signed with a digital certificate to be installable on devices. See the [official Tauri Android signing documentation](https://v2.tauri.app/distribute/sign/android/) for detailed instructions.

**Quick Summary:**

1. **Create a keystore** using `keytool` (PKCS12 format is recommended over JKS):
   ```bash
   keytool -genkey -v -keystore ~/upload-keystore.p12 -keyalg RSA -keysize 2048 -validity 10000 -alias upload -storetype PKCS12
   ```

2. **Create a keystore.properties file** at `src-tauri/gen/android/keystore.properties`:
   ```properties
   password=<password from keytool>
   keyAlias=upload
   storeFile=<path to upload-keystore.p12>
   ```
   > Keep both the keystore and properties file private; don't commit them to version control.

3. **Configure Gradle** in `src-tauri/gen/android/app/build.gradle.kts` to use the signing key for release builds.

For CI/CD environments (e.g., GitHub Actions), use secrets to securely pass the keystore and credentials.

#### Testing Mobile Screen Size on Desktop

To test responsive design for mobile during development without deploying to a device:

**Using Tauri Mobile Configuration:**
   - Edit `src-tauri/tauri.mobile.conf.json` to adjust the window size for testing
   - Run `pnpm dev:mobile` to test with mobile-specific settings
