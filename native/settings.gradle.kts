rootProject.name = "nomi"

enableFeaturePreview("TYPESAFE_PROJECT_ACCESSORS")

pluginManagement {
    includeBuild("build-logic")
    repositories {
        google {
            mavenContent {
                includeGroupAndSubgroups("androidx")
                includeGroupAndSubgroups("com.android")
                includeGroupAndSubgroups("com.google")
            }
        }
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google {
            mavenContent {
                includeGroupAndSubgroups("androidx")
                includeGroupAndSubgroups("com.android")
                includeGroupAndSubgroups("com.google")
            }
        }
        mavenCentral()
    }
}

include(
    ":shared:core",
    ":shared:designsystem",
    ":shared:network",
    ":shared:realtime",
    ":shared:database",
    ":shared:data",
    ":shared:feature:auth",
    ":shared:feature:home",
    ":shared:feature:chats",
    ":shared:feature:chat",
    ":shared:feature:notifications",
    ":shared:feature:settings",
    ":shared:companion",
    ":shared:app",
    ":androidApp",
    ":desktopApp",
)
