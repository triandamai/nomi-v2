import org.gradle.api.Project
import org.gradle.api.artifacts.VersionCatalog
import org.gradle.api.artifacts.VersionCatalogsExtension
import org.gradle.kotlin.dsl.getByType

internal val Project.libs: VersionCatalog
    get() = extensions.getByType<VersionCatalogsExtension>().named("libs")

internal fun VersionCatalog.versionInt(alias: String): Int =
    findVersion(alias).get().requiredVersion.toInt()

/** Package root shared by every module: `:shared:feature:auth` → `ai.nomi.feature.auth`. */
private const val PACKAGE_ROOT = "ai.nomi"
private const val SHARED_PREFIX = ":shared:"

internal val Project.nomiNamespace: String
    get() = PACKAGE_ROOT + "." + path.removePrefix(SHARED_PREFIX).replace(':', '.')
