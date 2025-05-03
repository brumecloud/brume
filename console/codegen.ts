import { type CodegenConfig } from "@graphql-codegen/cli";

const config: CodegenConfig = {
  schema: "/graph/public.graphql",
  documents: ["src/**/*.{ts,tsx,graphql.ts}"],
  generates: {
    "./src/_apollo/": {
      preset: "client",
      presetConfig: {
        gqlTagName: "gql",
      },
      config: {
        useTypeImports: true,
        inlineFragmentTypes: "mask",
        customDirectives: {
          apolloUnmask: true,
        },
      },
    },
  },
  ignoreNoDocuments: true,
};

export default config;
