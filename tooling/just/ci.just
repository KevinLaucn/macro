# Official Macro cloud deploys are disabled in this hard fork. Service
# justfiles still import this module, so keep the recipe as an explicit guard
# instead of removing the file and breaking local command parsing.
[no-cd]
deploy-dev SERVICE_NAME BRANCH="$(git branch --show-current)":
    @echo "Official Macro dev cloud deploy is disabled for service '{{SERVICE_NAME}}' on branch '{{BRANCH}}'. Use self-host image/deploy workflows instead."
    @exit 1
