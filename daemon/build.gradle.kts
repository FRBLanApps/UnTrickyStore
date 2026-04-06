plugins {
    id("base")
}

listOf("debug", "release").forEach { variantName ->
    val variantCapped = variantName.replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
    val variantLowered = variantName.lowercase()
    val profile = if (variantLowered == "release") "release" else "dev"
    val cargoFlag = if (variantLowered == "release") "--release" else ""

    tasks.register<Exec>("buildDaemon$variantCapped") {
        group = "rust"
        workingDir = projectDir
        commandLine("cargo", "ndk", "-t", "arm64-v8a", "-o", "target/aarch64-linux-android/$variantLowered", "build", *cargoFlag.split(" ").filter { it.isNotEmpty() }.toTypedArray())
        environment("CARGO_TARGET_DIR", "$projectDir/target_ndk")
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
