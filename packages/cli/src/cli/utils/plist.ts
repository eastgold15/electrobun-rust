import { basename, join } from "path";
import { existsSync } from "fs";

/**
 * Escape XML special characters
 */
export function escapeXml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

// Mapping of entitlements to their corresponding Info.plist usage description keys
const ENTITLEMENT_TO_PLIST_KEY: Record<string, string> = {
  "com.apple.security.device.camera": "NSCameraUsageDescription",
  "com.apple.security.device.microphone": "NSMicrophoneUsageDescription",
  "com.apple.security.device.audio-input": "NSMicrophoneUsageDescription",
  "com.apple.security.personal-information.location":
    "NSLocationUsageDescription",
  "com.apple.security.personal-information.location-when-in-use":
    "NSLocationWhenInUseUsageDescription",
  "com.apple.security.personal-information.contacts":
    "NSContactsUsageDescription",
  "com.apple.security.personal-information.calendars":
    "NSCalendarsUsageDescription",
  "com.apple.security.personal-information.reminders":
    "NSRemindersUsageDescription",
  "com.apple.security.personal-information.photos-library":
    "NSPhotoLibraryUsageDescription",
  "com.apple.security.personal-information.apple-music-library":
    "NSAppleMusicUsageDescription",
  "com.apple.security.personal-information.motion": "NSMotionUsageDescription",
  "com.apple.security.personal-information.speech-recognition":
    "NSSpeechRecognitionUsageDescription",
  "com.apple.security.device.bluetooth": "NSBluetoothAlwaysUsageDescription",
  "com.apple.security.files.user-selected.read-write":
    "NSDocumentsFolderUsageDescription",
  "com.apple.security.files.downloads.read-write":
    "NSDownloadsFolderUsageDescription",
  "com.apple.security.files.desktop.read-write":
    "NSDesktopFolderUsageDescription",
};

/**
 * Generate usage description entries for Info.plist from entitlements
 */
export function generateUsageDescriptions(
  entitlements: Record<string, boolean | string | string[]>,
): string {
  const usageEntries: string[] = [];

  for (const [entitlement, value] of Object.entries(entitlements)) {
    const plistKey = ENTITLEMENT_TO_PLIST_KEY[entitlement];
    if (plistKey && value) {
      const description =
        typeof value === "string"
          ? escapeXml(value)
          : `This app requires access for ${entitlement.split(".").pop()?.replace("-", " ")}`;

      usageEntries.push(
        `    <key>${plistKey}</key>\n    <string>${description}</string>`,
      );
    }
  }

  return usageEntries.join("\n");
}

/**
 * Generate CFBundleURLTypes for custom URL schemes
 */
export function generateURLTypes(
  urlSchemes: string[] | undefined,
  identifier: string,
): string {
  if (!urlSchemes || urlSchemes.length === 0) {
    return "";
  }

  const schemesXml = urlSchemes
    .map((scheme) => `                <string>${escapeXml(scheme)}</string>`)
    .join("\n");

  return `    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>${escapeXml(identifier)}</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>CFBundleURLSchemes</key>
            <array>
${schemesXml}
            </array>
        </dict>
    </array>`;
}

/**
 * Generates CFBundleDocumentTypes and UTExportedTypeDeclarations for file associations.
 */
export function generateDocumentTypes(
  fileAssociations: Array<{
    ext: string[];
    name: string;
    role?: "Editor" | "Viewer" | "Shell" | "None";
    icon?: string;
  }> | undefined,
  projectRoot: string,
  appIdentifier: string,
): string {
  if (!fileAssociations || fileAssociations.length === 0) {
    return "";
  }

  const validAssociations = fileAssociations.filter((assoc) => {
    if (!assoc.ext || assoc.ext.length === 0) {
      console.log(
        `WARNING: fileAssociations entry "${assoc.name || "(unnamed)"}" has no extensions — skipping`,
      );
      return false;
    }
    if (!assoc.name) {
      console.log(
        `WARNING: fileAssociations entry with extensions [${assoc.ext.join(", ")}] has no name — skipping`,
      );
      return false;
    }
    return true;
  });

  if (validAssociations.length === 0) {
    return "";
  }

  // Clean extensions and warn about leading dots
  const cleaned = validAssociations.map((assoc) => ({
    ...assoc,
    ext: assoc.ext.map((ext) => {
      const clean = ext.replace(/^\./, "");
      if (clean !== ext) {
        console.log(
          `WARNING: fileAssociations ext "${ext}" has a leading dot — stripping to "${clean}"`,
        );
      }
      return clean;
    }),
  }));

  // Generate CFBundleDocumentTypes with LSItemContentTypes
  const docTypes = cleaned
    .map((assoc) => {
      const role = assoc.role || "Viewer";
      let iconName = "";
      if (assoc.icon) {
        const iconSourcePath = join(projectRoot, assoc.icon);
        if (existsSync(iconSourcePath)) {
          iconName = basename(assoc.icon).replace(/\.icns$/i, "");
        } else {
          console.log(
            `WARNING: Document type icon not found: ${iconSourcePath} — skipping icon reference`,
          );
        }
      }
      const iconLine = iconName
        ? `            <key>CFBundleTypeIconFile</key>\n            <string>${escapeXml(iconName)}</string>\n`
        : "";
      const utiXml = assoc.ext
        .map(
          (ext) =>
            `                <string>${escapeXml(appIdentifier)}.${escapeXml(ext)}</string>`,
        )
        .join("\n");
      const extsXml = assoc.ext
        .map(
          (ext) =>
            `                <string>${escapeXml(ext)}</string>`,
        )
        .join("\n");

      return `        <dict>
            <key>CFBundleTypeName</key>
            <string>${escapeXml(assoc.name)}</string>
            <key>CFBundleTypeRole</key>
            <string>${escapeXml(role)}</string>
${iconLine}            <key>LSItemContentTypes</key>
            <array>
${utiXml}
            </array>
            <key>CFBundleTypeExtensions</key>
            <array>
${extsXml}
            </array>
        </dict>`;
    })
    .join("\n");

  // Generate UTExportedTypeDeclarations
  const utiDecls = cleaned
    .flatMap((assoc) => {
      let iconName = "";
      if (assoc.icon) {
        const iconSourcePath = join(projectRoot, assoc.icon);
        if (existsSync(iconSourcePath)) {
          iconName = basename(assoc.icon).replace(/\.icns$/i, "");
        }
      }
      const iconLine = iconName
        ? `            <key>UTTypeIconFiles</key>
            <array>
                <string>${escapeXml(iconName)}</string>
            </array>\n`
        : "";
      return assoc.ext.map(
        (ext) => `        <dict>
            <key>UTTypeIdentifier</key>
            <string>${escapeXml(appIdentifier)}.${escapeXml(ext)}</string>
            <key>UTTypeDescription</key>
            <string>${escapeXml(assoc.name)}</string>
            <key>UTTypeConformsTo</key>
            <array>
                <string>public.data</string>
            </array>
${iconLine}            <key>UTTypeTagSpecification</key>
            <dict>
                <key>public.filename-extension</key>
                <array>
                    <string>${escapeXml(ext)}</string>
                </array>
            </dict>
        </dict>`,
      );
    })
    .join("\n");

  return `    <key>CFBundleDocumentTypes</key>
    <array>
${docTypes}
    </array>
    <key>UTExportedTypeDeclarations</key>
    <array>
${utiDecls}
    </array>`;
}

