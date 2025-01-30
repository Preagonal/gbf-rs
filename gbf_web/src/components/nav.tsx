"use client";

import React, { useEffect, useState, useTransition } from "react";
import {
    Box,
    Title,
    Button,
    Select,
    Loader,
    Alert,
    ActionIcon,
    rem,
    Tooltip,
} from "@mantine/core";
import { useRouter, usePathname } from "next/navigation";
import { IconAlertCircle, IconBook, IconTrash } from "@tabler/icons-react";
import "./nav.css"; // Import the CSS file
import Link from "next/link";
import { DOCS_LINK } from "@/consts";

interface GbfVersion {
    gbfVersion: string;
}

interface GbfModule {
    moduleId: string;
    fileName: string;
}

interface GbfFunction {
    functionAddress: number;
    functionName: string;
}

export function NavigationBar() {
    // ---------- STATE ----------
    const [versions, setVersions] = useState<GbfVersion[]>([]);
    const [modules, setModules] = useState<GbfModule[]>([]);
    const [functions, setFunctions] = useState<GbfFunction[]>([]);

    // Loading indicators
    const [versionsLoading, setVersionsLoading] = useState(false);
    const [modulesLoading, setModulesLoading] = useState(false);
    const [functionsLoading, setFunctionsLoading] = useState(false);

    // Error states
    const [versionsError, setVersionsError] = useState<string | null>(null);
    const [modulesError, setModulesError] = useState<string | null>(null);
    const [functionsError, setFunctionsError] = useState<string | null>(null);

    // ---------- ROUTE PARSING ----------
    const pathname = usePathname();
    const segments = pathname.split("/").filter(Boolean);

    // e.g.  /0.1.2/foo/123 => ["0.1.2", "foo", "123"]
    const currentVersion = segments[0] ?? "";
    const currentModule = segments[1] ?? "";
    const currentFunction = segments[2] ?? "";

    const router = useRouter();

    const [isPending, startTransition] = useTransition();

    // Helper: build route based on version/module/function
    function createNavLink(version?: string, mod?: string, func?: string) {
        let path = "/";
        if (version) path += version;
        if (mod) path += `/${mod}`;
        if (func) path += `/${func}`;
        return path;
    }

    // Helper: "Back" link
    let backLink = "/";
    if (currentFunction) {
        backLink = createNavLink(currentVersion, currentModule);
    } else if (currentModule) {
        backLink = createNavLink(currentVersion);
    } else if (currentVersion) {
        backLink = "/";
    }

    // ---------- FETCH VERSIONS (ONCE) ----------
    useEffect(() => {
        setVersionsLoading(true);
        setVersionsError(null);

        fetch("/api/versions")
            .then((res) => {
                if (!res.ok) {
                    throw new Error(`Failed to fetch versions: ${res.status}`);
                }
                return res.json();
            })
            .then((data: GbfVersion[]) => setVersions(data))
            .catch((err) => {
                console.error(err);
                setVersionsError(err.message || "Error fetching versions");
            })
            .finally(() => setVersionsLoading(false));
    }, []);

    // ---------- FETCH MODULES WHEN version CHANGES ----------
    useEffect(() => {
        if (!currentVersion) {
            setModules([]);
            setModulesError(null);
            setFunctions([]);
            setFunctionsError(null);
            return;
        }

        setModulesLoading(true);
        setModulesError(null);

        fetch(`/api/modules?version=${currentVersion}`)
            .then((res) => {
                if (!res.ok) {
                    throw new Error(`Failed to fetch modules: ${res.status}`);
                }
                return res.json();
            })
            .then((data: GbfModule[]) => setModules(data))
            .catch((err) => {
                console.error(err);
                setModulesError(err.message || "Error fetching modules");
            })
            .finally(() => setModulesLoading(false));

        // Reset functions
        setFunctions([]);
        setFunctionsError(null);
    }, [currentVersion]);

    // ---------- FETCH FUNCTIONS WHEN (version, module) CHANGES ----------
    useEffect(() => {
        if (!currentVersion || !currentModule) {
            setFunctions([]);
            setFunctionsError(null);
            return;
        }

        setFunctionsLoading(true);
        setFunctionsError(null);

        fetch(`/api/functions?version=${currentVersion}&module=${currentModule}`)
            .then((res) => {
                if (!res.ok) {
                    throw new Error(`Failed to fetch functions: ${res.status}`);
                }
                return res.json();
            })
            .then((data: GbfFunction[]) => setFunctions(data))
            .catch((err) => {
                console.error(err);
                setFunctionsError(err.message || "Error fetching functions");
            })
            .finally(() => setFunctionsLoading(false));
    }, [currentVersion, currentModule]);

    // ---------- RENDER ----------
    return (
        <Box className="navbar-container">
            <div className="navbar-flex">
                <Title order={3} className="navbar-title">
                    GBF
                </Title>

                {/* VERSION SELECT */}
                <Select
                    data={versions.map((v) => ({ value: v.gbfVersion, label: v.gbfVersion }))}
                    placeholder="Select version"
                    value={currentVersion}
                    onChange={(value) =>
                        startTransition(() => router.push(createNavLink(value || undefined, currentModule, currentFunction)))
                    }
                    rightSection={versionsLoading || isPending ? <Loader size="xs" /> : null}
                    disabled={versionsLoading || isPending}
                    className="navbar-version"
                    searchable
                />

                {/* MODULE SELECT */}
                <Select
                    data={modules.map((m) => ({ value: m.moduleId, label: m.fileName }))}
                    placeholder="Select module"
                    value={currentModule}
                    onChange={(value) =>
                        startTransition(() => router.push(createNavLink(currentVersion, value || undefined, currentFunction)))
                    }
                    rightSection={modulesLoading || isPending ? <Loader size="xs" /> : null}
                    disabled={modulesLoading || isPending || !modules.length || !!currentFunction}
                    className="navbar-module"
                    searchable
                />

                {/* FUNCTION SELECT */}
                <Select
                    data={functions.map((f) => ({
                        value: f.functionAddress.toString(),
                        label: f.functionName || "[entrypoint]",
                    }))}
                    placeholder="Select function"
                    value={currentFunction}
                    onChange={(value) =>
                        startTransition(() =>
                            router.push(createNavLink(currentVersion, currentModule, value || undefined))
                        )
                    }
                    rightSection={functionsLoading || isPending ? <Loader size="xs" /> : null}
                    disabled={functionsLoading || isPending || !functions.length}
                    className="navbar-function"
                    searchable
                />

                {/* BACK BUTTON */}
                <Button onClick={() => startTransition(() => router.push(backLink))}>
                    Back
                </Button>

                {/* DOCS LINK */}
                <Tooltip label="Documentation" position="bottom" withArrow>
                    <ActionIcon
                        color="blue"
                        size={36}
                        radius="xl"
                        component={Link}
                        href={DOCS_LINK}
                        target="_blank"
                    >
                        <IconBook style={{ width: rem(16), height: rem(16) }} />
                    </ActionIcon>
                </Tooltip>
                <Tooltip label="Invalidate Cache" position="bottom" withArrow>
                    <ActionIcon
                        color="blue"
                        size={36}
                        radius="xl"
                        onClick={() => {
                            startTransition(async () => {
                                try {
                                    const response = await fetch("/api/cache", { method: "POST" });
                                    if (!response.ok) {
                                        throw new Error(`Failed to invalidate cache: ${response.status}`);
                                    }
                                    window.location.reload();
                                } catch (error) {
                                    console.error(error);
                                    alert("Error invalidating cache");
                                }
                            });
                        }}
                    >
                        <IconTrash style={{ width: rem(16), height: rem(16) }} />
                    </ActionIcon>
                </Tooltip>
            </div>

            {/* Error Alerts */}
            {versionsError && (
                <Alert
                    icon={<IconAlertCircle size="1rem" />}
                    title="Version Error"
                    color="red"
                    mt="sm"
                >
                    {versionsError}
                </Alert>
            )}
            {modulesError && (
                <Alert
                    icon={<IconAlertCircle size="1rem" />}
                    title="Module Error"
                    color="red"
                    mt="sm"
                >
                    {modulesError}
                </Alert>
            )}
            {functionsError && (
                <Alert
                    icon={<IconAlertCircle size="1rem" />}
                    title="Function Error"
                    color="red"
                    mt="sm"
                >
                    {functionsError}
                </Alert>
            )}
        </Box>
    );
}
