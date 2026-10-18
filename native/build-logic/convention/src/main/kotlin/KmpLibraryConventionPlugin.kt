import com.android.build.api.dsl.KotlinMultiplatformAndroidLibraryTarget
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.plugins.ExtensionAware
import org.gradle.kotlin.dsl.configure
import org.jetbrains.kotlin.gradle.dsl.KotlinMultiplatformExtension

/**
 * A Kotlin Multiplatform library for every Nomi target family:
 * Android, iOS (device + Apple-silicon simulator) and desktop JVM.
 */
class KmpLibraryConventionPlugin : Plugin<Project> {
    override fun apply(target: Project) = with(target) {
        pluginManager.apply("org.jetbrains.kotlin.multiplatform")
        pluginManager.apply("com.android.kotlin.multiplatform.library")

        extensions.configure<KotlinMultiplatformExtension> {
            jvmToolchain(libs.versionInt("jvm-toolchain"))

            (this as ExtensionAware).extensions.configure<KotlinMultiplatformAndroidLibraryTarget>(ANDROID_TARGET) {
                namespace = nomiNamespace
                compileSdk = libs.versionInt("android-compileSdk")
                minSdk = libs.versionInt("android-minSdk")
                withHostTest {}
            }

            iosArm64()
            iosSimulatorArm64()
            jvm(DESKTOP_TARGET)

            sourceSets.commonTest.dependencies {
                implementation(kotlin("test"))
            }
        }
    }

    private companion object {
        const val ANDROID_TARGET = "android"
        const val DESKTOP_TARGET = "desktop"
    }
}
