plugins {
    id("base")
}

val moduleId: String by rootProject.extra
val moduleName: String by rootProject.extra
val verName: String by rootProject.extra
val verType: String by rootProject.extra
val verCode: Int by rootProject.extra
val verHash: String by rootProject.extra

listOf("debug", "release").forEach { variantName ->
    val variantCapped = variantName.replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
    val variantLowered = variantName.lowercase()
    val moduleDir = layout.buildDirectory.dir("outputs/module/$variantLowered")
    val moduleOutputDir = moduleDir.get().asFile

    val prepareModuleFilesTask = tasks.register<Copy>("prepareModuleFiles$variantCapped") {
        group = "module"

        dependsOn(
            ":app:assemble$variantCapped",
            ":daemon:buildDaemon$variantCapped"
        )
        doFirst {
            with(moduleOutputDir) {
                deleteRecursively()
            }
        }
        into(moduleDir)
            from(project(":app").layout.buildDirectory.file("outputs/apk/$variantLowered")) {
                include(
                    "app-$variantLowered.apk"
                )
                rename(
                    "app-$variantLowered.apk",
                    "service.apk"
                )
            }
            from("$projectDir/src") {
                include(
                    "module.prop"
                )
                expand(
                    "moduleId" to "$moduleId",
                    "moduleName" to "$moduleName",
                    "versionName" to "$verName$verType ($verCode-$verHash-$variantLowered)",
                    "versionCode" to "$verCode"
                )
            }
            from("$projectDir/src") {
                exclude(
                    ".DS_Store",
                    "module.prop"
                )
            }
        into("bin") {
            from(project(":daemon").file("target/aarch64-linux-android/$variantLowered"))
            include("uts")
        }
    }

    tasks.register<Zip>("zip$variantCapped") {
        group = "module"
        dependsOn(prepareModuleFilesTask)
        archiveFileName.set("$moduleName-$verName-$verCode-$verHash-$variantName.zip".replace(' ', '-'))
        destinationDirectory.set(layout.buildDirectory.file("outputs/$variantLowered").get().asFile)
        from(moduleDir)
    }
}

tasks.register("zip") {
    group = "module"
    dependsOn(
        "zipDebug",
        "zipRelease"
    )
}
