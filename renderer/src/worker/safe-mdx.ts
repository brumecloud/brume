import { visit } from "unist-util-visit";
import { isAllowedMdxElement } from "../components";

type Position = {
  start?: { line?: number; column?: number };
};

type MdxAttribute = {
  type?: string;
  name?: string;
  value?: unknown;
};

type Node = {
  type?: string;
  name?: string | null;
  attributes?: MdxAttribute[];
  position?: Position;
};

const forbiddenNodeTypes = new Set([
  "mdxjsEsm",
  "mdxFlowExpression",
  "mdxTextExpression",
]);

function unsafe(node: Node, message: string): never {
  const error = new Error(message) as Error & {
    line?: number;
    column?: number;
  };
  error.line = node.position?.start?.line;
  error.column = node.position?.start?.column;
  throw error;
}

export function remarkSafeMdx() {
  return (tree: Node) => {
    visit(tree as never, (node: Node) => {
      if (node.type && forbiddenNodeTypes.has(node.type)) {
        unsafe(node, "JavaScript expressions, imports, and exports are disabled in safe MDX");
      }

      if (node.type === "mdxJsxFlowElement" || node.type === "mdxJsxTextElement") {
        if (!isAllowedMdxElement(node.name)) {
          unsafe(node, `MDX component \`${node.name ?? "fragment"}\` is not allowed`);
        }
        for (const attribute of node.attributes ?? []) {
          if (attribute.type !== "mdxJsxAttribute" || typeof attribute.value === "object") {
            unsafe(node, "MDX spread attributes and JavaScript attribute expressions are disabled");
          }
        }
      }
    });
  };
}

