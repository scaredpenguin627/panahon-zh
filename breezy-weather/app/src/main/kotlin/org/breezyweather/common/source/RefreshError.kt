/*
 * This file is part of Breezy Weather.
 *
 * Breezy Weather is free software: you can redistribute it and/or modify it
 * under the terms of the GNU Lesser General Public License as published by the
 * Free Software Foundation, version 3 of the License.
 *
 * Breezy Weather is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
 * or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
 * License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with Breezy Weather. If not, see <https://www.gnu.org/licenses/>.
 */

package org.breezyweather.common.source

import android.app.Activity
import android.content.Context
import androidx.annotation.StringRes
import breezyweather.domain.source.SourceFeature
import org.breezyweather.R
import org.breezyweather.common.utils.helpers.IntentHelper
import org.breezyweather.domain.source.resourceName
import org.breezyweather.sources.SourceManager
import org.breezyweather.ui.main.utils.RefreshErrorType

class RefreshError(
    val error: RefreshErrorType,
    val source: String? = null,
    val feature: SourceFeature? = null,
    @StringRes private val operation: Int? = null,
) {
    fun getSourceWithOptionalFeature(context: Context, sourceManager: SourceManager): String? {
        return if (!source.isNullOrEmpty()) {
            val sourceName = sourceManager.getSource(source)?.name ?: source
            val featureName = feature?.resourceName ?: operation
            if (featureName != null) {
                context.getString(R.string.parenthesis, sourceName, context.getString(featureName))
            } else {
                sourceName
            }
        } else {
            null
        }
    }

    fun getDescription(context: Context, sourceManager: SourceManager): String {
        if (error == RefreshErrorType.API_KEY_REQUIRED_MISSING && source != null) {
            val provider = sourceManager.getSource(source)?.name ?: source
            return context.getString(R.string.provider_key_missing, provider)
        }
        return context.getString(error.shortMessage)
    }

    fun actionLabel() = if (error == RefreshErrorType.API_KEY_REQUIRED_MISSING && source != null) {
        R.string.action_settings
    } else {
        error.actionButtonMessage
    }

    fun openHelp(activity: Activity, sourceManager: SourceManager) {
        val provider = source?.let { sourceManager.getSource(it) }
        if (error == RefreshErrorType.API_KEY_REQUIRED_MISSING && provider is ConfigurableSource) {
            if (provider is LocationSource) {
                IntentHelper.startLocationProviderSettingsActivity(activity)
            } else {
                IntentHelper.startWeatherProviderSettingsActivity(activity, provider.id)
            }
        } else {
            error.showDialogAction?.invoke(activity)
        }
    }

    fun getMessage(context: Context, sourceManager: SourceManager): String {
        return if (!source.isNullOrEmpty()) {
            "${getSourceWithOptionalFeature(context, sourceManager)}${context.getString(
                R.string.colon_separator
            )}${getDescription(context, sourceManager)}"
        } else {
            context.getString(error.shortMessage)
        }
    }
}
