import { analyze_diff } from "diff-effect-rust";
import { reversePatch, parsePatch, applyPatch } from "diff";

export interface DiffResult {
  name: string;
  chagne: "Added" | "Removed" | "Modified";
}


export const analyzeDiff = (newCode: string, patch: string):DiffResult[] => {
  const structured = parsePatch(patch);
  const reversed = reversePatch(structured);

  let result: string | false = newCode;

  for (const singlePatch of reversed) {
    if (result) {
      result = applyPatch(result, singlePatch);
    }
  }

  if (!result) {
    return [] as DiffResult[];
  }

  const oldCode = result;
  const diff = analyze_diff(oldCode, newCode) as DiffResult[];
  return diff;
};

export const analyzeDiffWithCode = (
  oldCode: string,
  newCode: string
) => {
  const diff = analyze_diff(oldCode, newCode) as DiffResult[];
  return diff;
};
