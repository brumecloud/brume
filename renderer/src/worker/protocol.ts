export type RenderRequest = {
  source_dir: string;
  output_dir: string;
  entry?: string;
  title?: string;
};

export type RenderSuccess = {
  ok: true;
  manifest_path: string;
  page_count: number;
  asset_count: number;
};

export type RenderFailure = {
  ok: false;
  message: string;
  file?: string;
  line?: number;
  column?: number;
};

export type RenderResponse = RenderSuccess | RenderFailure;

export type PageManifest = {
  route: string;
  object_path: string;
  source_path: string;
  title: string;
};

export type FileManifest = {
  path: string;
  sha256: string;
  size: number;
};

export type AssetManifest = FileManifest & {
  content_type: string;
};

export type BundleManifest = {
  format_version: 1;
  renderer_version: string;
  html_contract_version: 1;
  title: string;
  entry: string;
  pages: PageManifest[];
  assets: AssetManifest[];
  sources: FileManifest[];
};

