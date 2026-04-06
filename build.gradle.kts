val moduleId by extra("untrickystore")
val moduleName by extra("UnTrickyStore")
val verName by extra("v0.1.0")
val verType by extra("-alpha")
val verCode by extra(
    providers.exec {
        commandLine("git", "rev-list", "HEAD", "--count")
    }.standardOutput.asText.get().trim().toInt()
)
val verHash by extra(
    providers.exec {
        commandLine("git", "rev-parse", "--verify", "--short", "HEAD")
    }.standardOutput.asText.get().trim()
)
