/**
 * Generates only the TypeScript client from the Codama IDL.
 * Used by the Vercel build pipeline (no Rust toolchain required).
 */

import type { AnchorIdl } from '@codama/nodes-from-anchor';
import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';
import fs from 'fs';
import path from 'path';

import { createRewardsCodamaBuilder } from './lib/rewards-codama-builder';
import { preserveConfigFiles } from './lib/utils';

const projectRoot = path.join(__dirname, '..');
const idlDir = path.join(projectRoot, 'idl');
const rewardsIdl = JSON.parse(fs.readFileSync(path.join(idlDir, 'rewards_program.json'), 'utf-8')) as AnchorIdl;
const typescriptClientsDir = path.join(__dirname, '..', 'clients', 'typescript');

const rewardsCodama = createRewardsCodamaBuilder(rewardsIdl)
    .appendAccountDiscriminator()
    .appendAccountVersion()
    .appendPdaDerivers()
    .setInstructionAccountDefaultValues()
    .updateInstructionBumps()
    .removeEmitInstruction()
    .build();

const configPreserver = preserveConfigFiles(typescriptClientsDir);

async function main() {
    try {
        await Promise.resolve(
            rewardsCodama.accept(
                renderJavaScriptVisitor(path.join(typescriptClientsDir, 'src', 'generated'), {
                    deleteFolderBeforeRendering: true,
                    formatCode: true,
                }),
            ),
        );
    } finally {
        configPreserver.restore();
    }
}

void main();
