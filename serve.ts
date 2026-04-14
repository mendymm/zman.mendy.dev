import { serve } from "bun";
import { readFileSync, existsSync } from "fs";
import { join } from "path";

const PUBLIC_DIR = join(import.meta.dir, "public");
const HEADERS_FILE = join(PUBLIC_DIR, "_headers");

interface HeaderRule {
  pattern: RegExp;
  headers: Record<string, string>;
}

function parseHeadersFile(path: string): HeaderRule[] {
  if (!existsSync(path)) return [];

  const content = readFileSync(path, "utf-8");
  const lines = content.split("\n");
  const rules: HeaderRule[] = [];

  let currentPattern: string | null = null;
  let currentHeaders: Record<string, string> = {};

  for (const line of lines) {
    const trimmed = line.trim();

    if (!trimmed) {
      if (currentPattern) {
        rules.push({
          pattern: patternToRegex(currentPattern),
          headers: { ...currentHeaders },
        });
        currentPattern = null;
        currentHeaders = {};
      }
      continue;
    }

    if (!line.startsWith(" ") && !line.startsWith("\t")) {
      if (currentPattern) {
        rules.push({
          pattern: patternToRegex(currentPattern),
          headers: { ...currentHeaders },
        });
      }
      currentPattern = trimmed;
      currentHeaders = {};
    } else {
      const [key, ...valueParts] = trimmed.split(":");
      if (key && valueParts.length) {
        currentHeaders[key.trim()] = valueParts.join(":").trim();
      }
    }
  }

  if (currentPattern) {
    rules.push({
      pattern: patternToRegex(currentPattern),
      headers: { ...currentHeaders },
    });
  }

  return rules;
}

function patternToRegex(pattern: string): RegExp {
  let regex = pattern
    .replace(/\./g, "\\.")
    .replace(/\*/g, ".*");
  
  if (!regex.startsWith("/")) {
    regex = "/" + regex;
  }
  
  regex = "^" + regex + "$";
  return new RegExp(regex);
}

function getMatchingHeaders(path: string, rules: HeaderRule[]): Record<string, string> {
  const headers: Record<string, string> = {};

  for (const rule of rules) {
    if (rule.pattern.test(path)) {
      Object.assign(headers, rule.headers);
    }
  }

  return headers;
}

const headerRules = parseHeadersFile(HEADERS_FILE);
console.log(`Loaded ${headerRules.length} header rules`);

serve({
  port: 3000,
  async fetch(req) {
    const url = new URL(req.url);
    let path = url.pathname;

    if (path === "/") {
      path = "/index.html";
    }

    const filePath = join(PUBLIC_DIR, path);

    if (!existsSync(filePath)) {
      return new Response("Not Found", { status: 404 });
    }

    const file = Bun.file(filePath);
    const headers = getMatchingHeaders(path, headerRules);

    return new Response(file, {
      headers: {
        ...headers,
      },
    });
  },
});

console.log("Server running at http://localhost:3000");
