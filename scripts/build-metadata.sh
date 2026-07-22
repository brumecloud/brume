#!/usr/bin/env bash

brume_workspace_version() {
  awk '
    $0 == "[workspace.package]" {
      in_workspace_package = 1
      next
    }
    in_workspace_package && /^\[/ {
      exit
    }
    in_workspace_package && /^[[:space:]]*version[[:space:]]*=/ {
      value = $0
      sub(/^[^=]*=[[:space:]]*"/, "", value)
      sub(/"[[:space:]]*$/, "", value)
      print value
      exit
    }
  ' "${1:--}"
}

brume_validate_semver() {
  local version="$1"
  local semver_pattern='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-((0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*)(\.(0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*))*))?(\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$'

  [[ "${version}" =~ ${semver_pattern} ]]
}

brume_semver_is_greater() {
  local candidate="${1%%+*}"
  local previous="${2%%+*}"
  local candidate_core="${candidate%%-*}"
  local previous_core="${previous%%-*}"
  local candidate_prerelease=""
  local previous_prerelease=""
  local candidate_parts=()
  local previous_parts=()
  local candidate_identifiers=()
  local previous_identifiers=()
  local index

  [[ "${candidate}" == *-* ]] && candidate_prerelease="${candidate#*-}"
  [[ "${previous}" == *-* ]] && previous_prerelease="${previous#*-}"
  IFS=. read -r -a candidate_parts <<<"${candidate_core}"
  IFS=. read -r -a previous_parts <<<"${previous_core}"

  for index in 0 1 2; do
    if ((10#${candidate_parts[index]} > 10#${previous_parts[index]})); then
      return 0
    fi
    if ((10#${candidate_parts[index]} < 10#${previous_parts[index]})); then
      return 1
    fi
  done

  if [[ -z "${candidate_prerelease}" && -n "${previous_prerelease}" ]]; then
    return 0
  fi
  if [[ -n "${candidate_prerelease}" && -z "${previous_prerelease}" ]]; then
    return 1
  fi
  if [[ -z "${candidate_prerelease}" ]]; then
    return 1
  fi

  IFS=. read -r -a candidate_identifiers <<<"${candidate_prerelease}"
  IFS=. read -r -a previous_identifiers <<<"${previous_prerelease}"
  for ((index = 0; index < ${#candidate_identifiers[@]} || index < ${#previous_identifiers[@]}; index++)); do
    if ((index >= ${#candidate_identifiers[@]})); then
      return 1
    fi
    if ((index >= ${#previous_identifiers[@]})); then
      return 0
    fi

    local candidate_identifier="${candidate_identifiers[index]}"
    local previous_identifier="${previous_identifiers[index]}"
    if [[ "${candidate_identifier}" == "${previous_identifier}" ]]; then
      continue
    fi
    if [[ "${candidate_identifier}" =~ ^[0-9]+$ && "${previous_identifier}" =~ ^[0-9]+$ ]]; then
      ((10#${candidate_identifier} > 10#${previous_identifier})) && return 0
      return 1
    fi
    if [[ "${candidate_identifier}" =~ ^[0-9]+$ ]]; then
      return 1
    fi
    if [[ "${previous_identifier}" =~ ^[0-9]+$ ]]; then
      return 0
    fi
    [[ "${candidate_identifier}" > "${previous_identifier}" ]] && return 0
    return 1
  done

  return 1
}

brume_validate_workspace_version() {
  local repository="$1"
  local comparison_ref
  local current_version
  local previous_version

  current_version="$(brume_workspace_version "${repository}/Cargo.toml")"
  if ! brume_validate_semver "${current_version}"; then
    echo "error: workspace version '${current_version}' is not valid SemVer" >&2
    return 1
  fi

  if [[ -n "$(git -C "${repository}" status --porcelain --untracked-files=normal)" ]]; then
    comparison_ref="HEAD"
  else
    comparison_ref="HEAD^"
  fi

  if ! git -C "${repository}" rev-parse --verify "${comparison_ref}" >/dev/null 2>&1; then
    return 0
  fi
  if ! git -C "${repository}" cat-file -e "${comparison_ref}:Cargo.toml" 2>/dev/null; then
    return 0
  fi

  previous_version="$(git -C "${repository}" show "${comparison_ref}:Cargo.toml" | brume_workspace_version)"
  if ! brume_validate_semver "${previous_version}"; then
    echo "error: parent workspace version '${previous_version}' is not valid SemVer" >&2
    return 1
  fi
  if ! brume_semver_is_greater "${current_version}" "${previous_version}"; then
    echo "error: workspace version ${current_version} must be greater than ${comparison_ref} version ${previous_version}" >&2
    return 1
  fi
}

brume_export_build_metadata() {
  local repository="$1"

  if ! git -C "${repository}" rev-parse --verify HEAD >/dev/null 2>&1; then
    echo "error: Git metadata is required for a local Brume build" >&2
    return 1
  fi

  BRUME_BUILD_COMMIT="$(git -C "${repository}" rev-parse HEAD)"
  BRUME_BUILD_COMMIT_TITLE="$(git -C "${repository}" log -1 --format=%s)"
  BRUME_BUILD_COMMIT_MESSAGE="$(git -C "${repository}" log -1 --format=%b)"
  export BRUME_BUILD_COMMIT BRUME_BUILD_COMMIT_TITLE BRUME_BUILD_COMMIT_MESSAGE
}

brume_prepare_build() {
  local repository="$1"

  brume_validate_workspace_version "${repository}"
  brume_export_build_metadata "${repository}"
}
