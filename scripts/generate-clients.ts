/**
 * Generates TypeScript and Rust clients from the Codama IDL.
 */

import type { AnchorIdl } from '@codama/nodes-from-anchor';
import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';
import { renderVisitor as renderRustVisitor } from '@codama/renderers-rust';
import fs from 'fs';
import path from 'path';

import { createRewardsCodamaBuilder } from './lib/rewards-codama-builder';
import { preserveConfigFiles } from './lib/utils';

const projectRoot = path.join(__dirname, '..');
const idlDir = path.join(projectRoot, 'idl');
const rewardsIdl = JSON.parse(fs.readFileSync(path.join(idlDir, 'rewards_program.json'), 'utf-8')) as AnchorIdl;
const rustClientsDir = path.join(__dirname, '..', 'clients', 'rust');
const typescriptClientsDir = path.join(__dirname, '..', 'clients', 'typescript');

const rewardsCodama = createRewardsCodamaBuilder(rewardsIdl)
    .appendAccountDiscriminator()
    .appendAccountVersion()
    .appendPdaDerivers()
    .setInstructionAccountDefaultValues()
    .updateInstructionBumps()
    .removeEmitInstruction()
    .build();

// Preserve configuration files during generation
const configPreserver = preserveConfigFiles(typescriptClientsDir, rustClientsDir);

async function main() {
    try {
        // Generate Rust client.
        await Promise.resolve(
            rewardsCodama.accept(
                renderRustVisitor(path.join(rustClientsDir, 'src', 'generated'), {
                    crateFolder: rustClientsDir,
                    deleteFolderBeforeRendering: true,
                    formatCode: true,
                }),
            ),
        );

        // Generate TypeScript client.
        await Promise.resolve(
            rewardsCodama.accept(
                renderJavaScriptVisitor(path.join(typescriptClientsDir, 'src', 'generated'), {
                    deleteFolderBeforeRendering: true,
                    formatCode: true,
                }),
            ),
        );
    } finally {
        // Restore configuration files after generation.
        configPreserver.restore();
    }
}

void main();
