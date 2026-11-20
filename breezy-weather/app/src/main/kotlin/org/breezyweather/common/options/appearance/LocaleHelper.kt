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

package org.breezyweather.common.options.appearance

import android.content.Context
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.toImmutableList
import org.breezyweather.R
import org.xmlpull.v1.XmlPullParser

object LocaleHelper {

    data class Language(val langTag: String, val displayName: String)

    fun getLangs(context: Context): ImmutableList<Language> = buildList {
        add(Language("", context.getString(R.string.settings_follow_system)))
        context.resources.getXml(R.xml.locales_config).use { parser ->
            while (parser.next() != XmlPullParser.END_DOCUMENT) {
                if (parser.eventType == XmlPullParser.START_TAG && parser.name == "locale") {
                    val tag = parser.getAttributeValue("http://schemas.android.com/apk/res/android", "name")
                    val label = when (tag) {
                        "en" -> "English"
                        "zh-CN" -> "简体中文"
                        "zh-TW" -> "繁體中文"
                        else -> continue
                    }
                    add(Language(tag, label))
                }
            }
        }
    }.toImmutableList()
}
