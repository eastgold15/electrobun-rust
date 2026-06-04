import { execSync } from "child_process";
import { existsSync, readdirSync, unlinkSync } from "fs";
import { basename, dirname, join } from "path";
import { OS, ARCH } from "../../@pori15/electrobun-rust/src/core/shared/platform";
import { escapePathForTerminal, buildEntitlementsFile } from "../utils/plist";
import { getCEFHelperNames } from "../utils/paths";

export function codesignAppBundle(
    appBundleOrDmgPath: string,
    entitlementsFilePath: string | undefined,
    config: any,
) {
    console.log("code signing...");
    if (OS !== "macos" || !config.build.mac.codesign) return;

    const ELECTROBUN_DEVELOPER_ID = process.env["ELECTROBUN_DEVELOPER_ID"];
    if (!ELECTROBUN_DEVELOPER_ID) {
        console.error("Env var ELECTROBUN_DEVELOPER_ID is required to codesign");
        process.exit(1);
    }

    if (appBundleOrDmgPath.endsWith(".dmg")) {
        execSync(
            `codesign --force --verbose --timestamp --sign "${ELECTROBUN_DEVELOPER_ID}" ${escapePathForTerminal(appBundleOrDmgPath)}`,
        );
        return;
    }

    const contentsPath = join(appBundleOrDmgPath, "Contents");
    const macosPath = join(contentsPath, "MacOS");

    if (entitlementsFilePath) {
        const entitlementsFileContents = buildEntitlementsFile(config.build.mac.entitlements);
        Bun.write(entitlementsFilePath, entitlementsFileContents);
    }

    const frameworksPath = join(contentsPath, "Frameworks");
    if (existsSync(frameworksPath)) {
        try {
            const frameworks = readdirSync(frameworksPath);
            for (const framework of frameworks) {
                if (framework.endsWith(".framework")) {
                    const frameworkPath = join(frameworksPath, framework);
                    if (framework === "Chromium Embedded Framework.framework") {
                        console.log(`Signing CEF framework components: ${framework}`);
                        const librariesPath = join(frameworkPath, "Libraries");
                        if (existsSync(librariesPath)) {
                            const libraries = readdirSync(librariesPath);
                            for (const library of libraries) {
                                if (library.endsWith(".dylib")) {
                                    const libraryPath = join(librariesPath, library);
                                    console.log(`Signing CEF library: ${library}`);
                                    execSync(
                                        `codesign --force --verbose --timestamp --sign "${ELECTROBUN_DEVELOPER_ID}" --options runtime ${escapePathForTerminal(libraryPath)}`,
                                    );
                                }
                            }
                        }
                    }
                    console.log(`Signing framework bundle: ${framework}`);
                    execSync(
                        `codesign --force --verbose --timestamp --sign "${ELECTROBUN_DEVELOPER_ID}" --options runtime ${escapePathForTerminal(frameworkPath)}`,
                    );
                }
            }
        } catch (err) {
            console.log("Error signing frameworks:", err);
            throw err;
        }
    }

    const cefHelperApps = getCEFHelperNames().map((name) => `${name}.app`);
    for (const helperApp of cefHelperApps) {
        const helperPath = join(frameworksPath, helperApp);
        if (existsSync(helperPath)) {
            const helperExecutablePath = join(
                helperPath, "Contents", "MacOS",
                helperApp.replace(".app", ""),
            );
            if (existsSync(helperExecutablePath)) {
                console.log(`Signing CEF helper executable: ${helperApp}`);
                const ef = entitlementsFilePath ? `--entitlements ${entitlementsFilePath}` : "";
                execSync(
                    `codesign --force --verbose --timestamp --sign "${ELECTROBUN_DEVELOPER_ID}" --options runtime ${ef} ${escapePathForTerminal(helperExecutablePath)}`,
                );
            }
            console.log(`Signing CEF helper bundle: ${helperApp}`);
            const ef = entitlementsFilePath ? `--entitlements ${entitlementsFilePath}` : "";
            execSync(
                `codesign --force --verbose --timestamp --sign "${ELECTROBUN_DEVELOPER_ID}" --options runtime ${ef} ${escapePathForTerminal(helperPath)}`,
            );
        }
    }

    console.log("Signing all binaries in MacOS folder...");
    function findExecutables(dir: string): string[] {
        let executables: string[] = [];
        try {
            const entries = readdirSync(dir, { withFileTypes: true });
            for (const entry of entries) {
                const fullPath = join(dir, entry.name);
                if (entry.isDirectory()) {
                    executables = executables.concat(findExecutables(fullPath));
                } else if (entry.isFile()) {
                    try {
                        const fileInfo = execSync(`file -b ${escapePathForTerminal(fullPath)}`, { encoding: "utf8" }).trim();
                        if (fileInfo.includes("Mach-O") || entry.name.endsWith(".dylib")) {
                            executables.push(fullPath);
                        }
                    } catch {
                        if (entry.name.endsWith(".dylib") || !entry.name.includes(".")) {
                            executables.push(fullPath);
                        }
                    }
                }
            }
        } catch (err) {
            console.error(`Error scanning directory ${dir}:`, err);
        }
        return executables;
    }

    const executablesInMacOS = findExecutables(macosPath);
    for (const execPath of executablesInMacOS) {
        const fileName = basename(execPath);
        const relativePath = execPath.replace(macosPath + "/", "");
        const identifier = fileName.replace(/\.[^.]+$/, "");
        console.log(`Signing ${relativePath} with identifier ${identifier}`);
        const ef = entitlementsFilePath ? `--entitlements ${entitlementsFilePath}` : "";
        try {
            execSync(
                `codesign --force --verbose --timestamp --sign "${ELECTROBUN_DEVELOPER_ID}" --options runtime --identifier ${identifier} ${ef} ${escapePathForTerminal(execPath)}`,
            );
        } catch (err) {
            console.error(`Failed to sign ${relativePath}:`, (err as Error).message);
        }
    }

    const resourcesPath = join(contentsPath, "Resources", "app", "bun");
    if (existsSync(resourcesPath)) {
        console.log("Signing native modules in Resources/app/bun...");
        try {
            const nodeFiles = execSync(`find ${escapePathForTerminal(resourcesPath)} -name "*.node" -type f`, { encoding: "utf8" })
                .trim().split("\n").filter(Boolean);
            for (const nodeFile of nodeFiles) {
                if (nodeFile) {
                    console.log(`Signing native module: ${nodeFile.replace(resourcesPath + "/", "")}`);
                    execSync(
                        `codesign --force --verbose --timestamp --sign "${ELECTROBUN_DEVELOPER_ID}" --options runtime ${escapePathForTerminal(nodeFile)}`,
                    );
                }
            }
        } catch (err) {
            console.error("Error signing native modules:", err);
        }
    }

    const launcherPath = join(macosPath, "launcher");
    if (existsSync(launcherPath)) {
        console.log("Signing main executable (launcher)");
        const ef = entitlementsFilePath ? `--entitlements ${entitlementsFilePath}` : "";
        try {
            execSync(
                `codesign --force --verbose --timestamp --sign "${ELECTROBUN_DEVELOPER_ID}" --options runtime ${ef} ${escapePathForTerminal(launcherPath)}`,
            );
        } catch (error) {
            console.error("Failed to sign launcher:", (error as Error).message);
            console.log("Attempting to sign launcher without runtime hardening...");
            execSync(
                `codesign --force --verbose --timestamp --sign "${ELECTROBUN_DEVELOPER_ID}" ${ef} ${escapePathForTerminal(launcherPath)}`,
            );
        }
    }

    console.log("Signing app bundle");
    const ef = entitlementsFilePath ? `--entitlements ${entitlementsFilePath}` : "";
    execSync(
        `codesign --force --verbose --timestamp --sign "${ELECTROBUN_DEVELOPER_ID}" --options runtime ${ef} ${escapePathForTerminal(appBundleOrDmgPath)}`,
    );
}

export function notarizeAndStaple(
    appOrDmgPath: string,
    config: any,
) {
    if (OS !== "macos" || !config.build.mac.notarize) return;

    console.log("notarizing...");
    const zipPath = appOrDmgPath + ".zip";
    const appBundleFileName = basename(appOrDmgPath);
    execSync(
        `zip -y -r -9 ${escapePathForTerminal(zipPath)} ${escapePathForTerminal(appBundleFileName)}`,
        { stdio: "inherit", cwd: dirname(appOrDmgPath) },
    );
    execSync(
        `xcrun notarytool submit ${escapePathForTerminal(zipPath)} --apple-id "${process.env.APPLE_ID || ""}" --team-id "${process.env.APPLE_TEAM_ID || ""}" --password "${process.env.APPLE_APP_SPECIFIC_PASSWORD || ""}" --wait`,
        { stdio: "inherit" },
    );
    console.log("stapling...");
    execSync(`xcrun stapler staple ${escapePathForTerminal(appOrDmgPath)}`, { stdio: "inherit" });
    if (existsSync(zipPath)) unlinkSync(zipPath);
}
