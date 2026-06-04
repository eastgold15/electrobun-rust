import { basename, dirname, join } from "path";
import {
  existsSync,
  mkdirSync,
  writeFileSync,
} from "fs";
import * as readline from "readline";
import { getTemplate, getTemplateNames } from "../../templates/embedded";

/**
 * Initialize a new Electrobun project from a template
 */
export async function initCommand() {
  const indexOfElectrobun = process.argv.findIndex((arg) =>
    arg.includes("electrobun"),
  );
  const secondArg = process.argv[indexOfElectrobun + 2];
  const availableTemplates = getTemplateNames();

  let projectName: string;
  let templateName: string;

  const templateFlag = process.argv.find((arg) =>
    arg.startsWith("--template="),
  );
  if (templateFlag) {
    projectName = secondArg || "my-electrobun-app";
    templateName = templateFlag.split("=")[1]!;
  } else if (secondArg && availableTemplates.includes(secondArg)) {
    projectName = secondArg;
    templateName = secondArg;
  } else {
    console.log("🚀 Welcome to Electrobun!");
    console.log("");
    console.log("Available templates:");
    availableTemplates.forEach((template, index) => {
      console.log(`  ${index + 1}. ${template}`);
    });
    console.log("");

    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });

    const choice = await new Promise<string>((resolve) => {
      rl.question("Select a template (enter number): ", (answer) => {
        rl.close();
        resolve(answer.trim());
      });
    });

    const templateIndex = parseInt(choice) - 1;
    if (templateIndex < 0 || templateIndex >= availableTemplates.length) {
      console.error(`❌ Invalid selection. Please enter a number between 1 and ${availableTemplates.length}.`);
      process.exit(1);
    }

    templateName = availableTemplates[templateIndex]!;

    const rl2 = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });

    projectName = await new Promise<string>((resolve) => {
      rl2.question(`Enter project name (default: my-${templateName}-app): `, (answer) => {
        rl2.close();
        resolve(answer.trim() || `my-${templateName}-app`);
      });
    });
  }

  console.log(`🚀 Initializing Electrobun project: ${projectName}`);
  console.log(`📋 Using template: ${templateName}`);

  if (!availableTemplates.includes(templateName)) {
    console.error(`❌ Template "${templateName}" not found.`);
    console.log(`Available templates: ${availableTemplates.join(", ")}`);
    process.exit(1);
  }

  const template = getTemplate(templateName);
  if (!template) {
    console.error(`❌ Could not load template "${templateName}"`);
    process.exit(1);
  }

  const projectPath = join(process.cwd(), projectName);
  if (existsSync(projectPath)) {
    console.error(`❌ Directory "${projectName}" already exists.`);
    process.exit(1);
  }

  mkdirSync(projectPath, { recursive: true });

  let fileCount = 0;
  for (const [relativePath, content] of Object.entries(template.files)) {
    const fullPath = join(projectPath, relativePath);
    const dir = dirname(fullPath);
    mkdirSync(dir, { recursive: true });

    if (content.startsWith("base64:")) {
      writeFileSync(fullPath, new Uint8Array(Buffer.from(content.slice(7), "base64")));
    } else {
      writeFileSync(fullPath, content, "utf-8");
    }
    fileCount++;
  }

  console.log(`✅ Created ${fileCount} files from "${templateName}" template`);
  console.log(`📁 Project created at: ${projectPath}`);
  console.log("");
  console.log("📦 Next steps:");
  console.log(`   cd ${projectName}`);
  console.log("   bun install");
  console.log("   bun start");
  console.log("");
  console.log("🎉 Happy building with Electrobun!");
}
