import type { ObjectFieldTemplateProps } from "@rjsf/utils";

export const ObjectFieldTemplate = (props: ObjectFieldTemplateProps) => {
  const { properties, description } = props;

  return (
    <div className="flex flex-col space-y-6">
      <div className="flex flex-col space-y-1">
        {description && <p className="text-gray-500 text-sm">{description}</p>}
      </div>
      <div className="flex flex-col space-y-4">
        {properties.map((property) => property.content)}
      </div>
    </div>
  );
};
