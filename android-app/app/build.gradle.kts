plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.serialization")
}

android {
    namespace = "org.panahon.zh"
    compileSdk = 35

    defaultConfig {
        applicationId = "org.panahon.zh"
        minSdk = 29
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"

        buildConfigField("String", "DEFAULT_API_BASE_URL", "\"http://10.0.2.2:8788\"")
        manifestPlaceholders["allowCleartext"] = "true"
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    buildTypes {
        debug {
            buildConfigField("boolean", "ALLOW_CLEARTEXT_API", "true")
        }
        release {
            isMinifyEnabled = false
            buildConfigField("String", "DEFAULT_API_BASE_URL", "\"\"")
            buildConfigField("boolean", "ALLOW_CLEARTEXT_API", "false")
            manifestPlaceholders["allowCleartext"] = "false"
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    packaging {
        resources.excludes += "/META-INF/{AL2.0,LGPL2.1}"
    }

    sourceSets["main"].jniLibs.srcDirs("src/main/jniLibs")
}

dependencies {
    implementation(platform("androidx.compose:compose-bom:2024.12.01"))
    implementation("androidx.activity:activity-compose:1.10.0")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-extended")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.8.7")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.8.7")
    implementation("androidx.lifecycle:lifecycle-viewmodel-ktx:2.8.7")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.9.0")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")
    implementation("androidx.core:core-ktx:1.15.0")
    debugImplementation("androidx.compose.ui:ui-tooling")
}

tasks.register("buildRustNative") {
    group = "native"
    description = "Build the optional Rust JNI library when cargo and Android targets are installed."
    doLast {
        if (providers.gradleProperty("withRust").orNull != "true") {
            logger.lifecycle("Skipping Rust JNI build. Use -PwithRust=true for a native release build.")
            return@doLast
        }
        val root = rootProject.projectDir.parentFile
        val cargo = providers.environmentVariable("CARGO").orElse("cargo").get()
        val requested = providers.environmentVariable("PANAHON_ANDROID_TARGET").orNull
        val targets = requested?.let { listOf(it) } ?: listOf("aarch64-linux-android", "x86_64-linux-android")
        targets.forEach { target ->
            exec {
                workingDir(root)
                commandLine(cargo, "build", "-p", "panahon-mobile-core", "--features", "jni", "--target", target, "--release")
            }
            val output = root.resolve("target/$target/release/libpanahon_mobile_core.so")
            val abi = if (target == "x86_64-linux-android") "x86_64" else "arm64-v8a"
            val destination = projectDir.resolve("src/main/jniLibs/$abi")
            destination.mkdirs()
            copy {
                from(output)
                into(destination)
            }
        }
    }
}

tasks.named("preBuild").configure {
    dependsOn("buildRustNative")
}

