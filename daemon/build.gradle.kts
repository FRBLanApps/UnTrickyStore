plugins {
    id("base")
}

fun findNdkLinker(): String {
    val sdkRoot = System.getenv("ANDROID_HOME")
        ?: System.getenv("ANDROID_SDK_ROOT")
        ?: throw GradleException("ANDROID_HOME or ANDROID_SDK_ROOT not set")
    val ndkDir = file("$sdkRoot/ndk").listFiles()
        ?.filter { it.isDirectory }
        ?.maxByOrNull { it.name }
        ?: throw GradleException("No NDK found in $sdkRoot/ndk")
    val hostTag = when {
        org.gradle.internal.os.OperatingSystem.current().isLinux -> "linux-x86_64"
        org.gradle.internal.os.OperatingSystem.current().isMacOsX -> "darwin-x86_64"
        org.gradle.internal.os.OperatingSystem.current().isWindows -> "windows-x86_64"
        else -> throw GradleException("Unsupported OS")
    }
    val cc = file("${ndkDir}/toolchains/llvm/prebuilt/$hostTag/bin/aarch64-linux-android24-clang")
    if (!cc.exists()) throw GradleException("NDK clang not found at $cc")
    return cc.absolutePath
}

listOf("debug", "release").forEach { variantName ->
    val variantCapped = variantName.replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
    val variantLowered = variantName.lowercase()
    val profile = if (variantLowered == "release") "release" else "debug"
    val cargoFlag = if (variantLowered == "release") "--release" else ""

    tasks.register<Exec>("buildDaemon$variantCapped") {
        group = "rust"
        workingDir = projectDir
        commandLine("cargo", "build", "--target", "aarch64-linux-android",
            *cargoFlag.split(" ").filter { it.isNotEmpty() }.toTypedArray())
        environment("CARGO_TARGET_DIR", "$projectDir/target_ndk")
        environment("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", findNdkLinker())
        doFirst {
            println("Building uts binary ($variantLowered)...")
        }
        doLast {
            val src = file("target_ndk/aarch64-linux-android/$profile/uts")
            val dst = file("target/aarch64-linux-android/$variantLowered/uts")
            dst.parentFile.mkdirs()
            src.copyTo(dst, overwrite = true)
            println("uts binary: ${dst.absolutePath}")
        }
    }
}
