#!/usr/bin/env node
/**
 * Build script for Python backend sidecar.
 * 
 * This script:
 * 1. Runs PyInstaller to create the backend executable
 * 2. Renames with target-triple suffix for Tauri
 * 3. Copies to src-tauri/binaries/
 */

import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, '..');
const backendDir = path.join(projectRoot, 'backend');
const binariesDir = path.join(projectRoot, 'src-tauri', 'binaries');

// Get platform-specific settings
const isWindows = process.platform === 'win32';
const isMac = process.platform === 'darwin';
const extension = isWindows ? '.exe' : '';

// Get the target triple from rustc or fall back to defaults
function getTargetTriple() {
    try {
        const rustInfo = execSync('rustc -vV', { encoding: 'utf-8' });
        const match = /host: (\S+)/.exec(rustInfo);
        if (match) {
            return match[1];
        }
    } catch (err) {
        console.warn('⚠️  rustc not found, using platform defaults');
    }

    // Fallback to common target triples based on platform
    if (isWindows) {
        if (process.arch === 'arm64') {
            return 'aarch64-pc-windows-msvc';
        }
        return 'x86_64-pc-windows-msvc';
    } else if (isMac) {
        if (process.arch === 'arm64') {
            return 'aarch64-apple-darwin';
        }
        return 'x86_64-apple-darwin';
    } else {
        // Linux
        if (process.arch === 'arm64') {
            return 'aarch64-unknown-linux-gnu';
        }
        return 'x86_64-unknown-linux-gnu';
    }
}

// Main build function
async function build() {
    console.log('🔨 Building Tremors Music backend executable...\n');

    // Ensure binaries directory exists
    if (!fs.existsSync(binariesDir)) {
        fs.mkdirSync(binariesDir, { recursive: true });
    }

    // Step 1: Sync dependencies (including dev)
    console.log('📦 Syncing dependencies...');
    execSync('uv sync --dev', {
        cwd: backendDir,
        stdio: 'inherit',
    });

    // Step 2: Run PyInstaller
    console.log('\n🐍 Running PyInstaller...');
    execSync('uv run pyinstaller backend.spec --clean --noconfirm', {
        cwd: backendDir,
        stdio: 'inherit',
    });

    // Step 3: Get target triple and rename binary
    const targetTriple = getTargetTriple();
    console.log(`\n🎯 Target triple: ${targetTriple}`);

    const sourcePath = path.join(backendDir, 'dist', `tremorsmusic${extension}`);
    const destPath = path.join(binariesDir, `tremorsmusic-${targetTriple}${extension}`);

    // Remove old binary if exists
    if (fs.existsSync(destPath)) {
        fs.unlinkSync(destPath);
    }

    // Copy to binaries folder with correct name
    console.log(`\n📁 Copying to ${destPath}...`);
    fs.copyFileSync(sourcePath, destPath);

    // Make executable on Unix
    if (!isWindows) {
        fs.chmodSync(destPath, 0o755);
    }

    console.log('\n✅ Tremors Music backend built successfully!');
    console.log(`   Location: ${destPath}`);
}

build().catch((err) => {
    console.error('❌ Build failed:', err.message);
    process.exit(1);
});

