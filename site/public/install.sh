#!/bin/sh
# brume.dev/install.sh, until the first release of Brume.
#
# This placeholder installs nothing: it says Brume is not out yet, and exits 1 so that a script
# or an AI agent running `curl -sSL https://brume.dev/install.sh | bash` sees it failed.
# On the first release, site/vercel.json redirects /install.sh to the script of the latest
# GitHub release again, and this file goes away: see site/README.md, "Hosting".

if [ -t 1 ]; then
  white=$(printf '\033[97m')
  blue=$(printf '\033[94m')
  dim=$(printf '\033[2m')
  bold=$(printf '\033[1m')
  reset=$(printf '\033[0m')
else
  white='' blue='' dim='' bold='' reset=''
fi

printf '\n%s' "$white"
cat <<'CLOUD'
                .-~~~-.
        .-~~~-./       \.-~~-.
       /                      \
    .-'                        '-.
   (                              )
    '-.________________________.-'
CLOUD
printf '%s' "$blue"
cat <<'MIST'
  ~  ~~~ ~   ~~  ~~~~  ~  ~~~  ~~ ~
     ~~ ~  ~~~~  ~ ~~ ~~~  ~~  ~
MIST
printf '%s\n' "$reset"
printf '  %sbrume%s %s· just push it%s\n\n' "$bold" "$reset" "$dim" "$reset"
printf '  Brume is not out yet: alpha 0.1 is in progress.\n'
printf '  Nothing was installed.\n\n'
printf '  Watch the repository to know when it ships:\n'
printf '  %shttps://github.com/brumecloud/brume%s\n\n' "$bold" "$reset"
printf '  %sMeanwhile, see how it works: https://brume.dev/architecture%s\n\n' "$dim" "$reset"

exit 1
