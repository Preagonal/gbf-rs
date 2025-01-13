import '@mantine/core/styles.css';
import '@mantine/code-highlight/styles.css';
import {
  Box,
  ColorSchemeScript,
  MantineProvider,
  mantineHtmlProps,
} from '@mantine/core';
import { createQueryBuilder } from '@/data/data-analyzer';
import { FunctionResult, ModuleResult } from '@/data/gbf-suite-result-dao';

export const metadata = {
  title: 'GBF Web',
  description: 'The GBF decomplier and CFG tracker',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" {...mantineHtmlProps}>
      <head>
        <ColorSchemeScript />
      </head>
      <body>
        <MantineProvider defaultColorScheme="dark">
          <Box>{children}</Box>
        </MantineProvider>
      </body>
    </html>
  );
}

export async function generateStaticParams() {
  const queryBuilder = await createQueryBuilder();
  const versions = queryBuilder.execute() as string[];

  // Safeguard against empty or undefined versions
  if (!Array.isArray(versions) || versions.length === 0) {
    throw new Error('No versions found in the analyzer.');
  }

  // Get all modules
  const modules = versions
    .map((version) => queryBuilder.version(version).execute() as ModuleResult[])
    .filter((moduleArray) => Array.isArray(moduleArray)) // Ensure it's iterable
    .flat();

  // Safeguard against empty or undefined modules
  if (!Array.isArray(modules) || modules.length === 0) {
    throw new Error('No modules found for the given versions.');
  }

  // Get all functions
  const functions = modules
    .map((module) => {
      const result = queryBuilder.version(module.moduleId.version).module(module.moduleId.moduleId).execute();
      return Array.isArray(result) ? result : [];
    })
    .flat() as FunctionResult[];

  return functions.map((func: FunctionResult) => ({
    version: func.functionId.version,
    module: func.functionId.moduleId,
    function: func.functionId.functionAddress.toString(10)
  }));
}

