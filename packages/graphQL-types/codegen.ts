import { CodegenConfig } from "@graphql-codegen/cli";

const config: CodegenConfig = {
  schema: "http://localhost:9877/graphql",
  documents: "../../console/src/**/*.ts?(x)",
  // this assumes that all your source files are in a top-level `src/` directory - you might need to adjust this to your file structure
  generates: {
    "./src/__generated__/": {
      preset: "client",
      plugins: [],
      presetConfig: {
        gqlTagName: "gql",
        useTypeImports: true,
      },
    },
  },
};

export default config;
