import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";
import { renderSnippet } from "$lib/components/ui/data-table/index.js";
import { renderComponent } from "$lib/components/ui/data-table/index.js";
import DataTableActions from "./data-table-actions.svelte";

import type { LibraryDirInfo } from "../libraryInfo.svelte";

export const columns: ColumnDef<LibraryDirInfo>[] = [
    {
        accessorKey: "dir",
        header: () => {
            const headerSnippet = createRawSnippet(() => ({
                render: () => `<div class="text-center flex-1">Directory</div>`,
            }));
            return renderSnippet(headerSnippet, "");
        },
        cell: ({ row }) => {
            const cellSnippet = createRawSnippet<[string]>((getData) => {
                const data = getData();
                return {
                    render: () =>
                        `<div class="ml-1.5 sm:ml-2 md:ml-4 lg:ml-6 h-fit text-left font-normal text-wrap wrap-anywhere break-all line-clamp-1 text-ellipsis" title="${data}">${data}</div>
                    `,
                };
            });

            return renderSnippet(cellSnippet, row.getValue("dir"));
        },
    },
    {
        accessorKey: "fileCount",
        header: () => {
            const headerSnippet = createRawSnippet(() => ({
                render: () => `<div class="text-center flex-1">Files</div>`,
            }));
            return renderSnippet(headerSnippet, "");
        },
        cell: ({ row }) => {
            const cellSnippet = createRawSnippet<[string]>((getData) => {
                const data = getData();
                return {
                    render: () =>
                        `<div class="h-fit text-center font-normal text-wrap line-clamp-1 text-ellipsis" title="${data}">${data}</div>`,
                };
            });

            return renderSnippet(cellSnippet, row.getValue("fileCount"));
        },
    },
    {
        id: "actions",
        cell: ({ row }) => {
            // You can pass whatever you need from `row.original` to the component
            return renderComponent(DataTableActions, { path: row.original.dir });
        },
    },
];
