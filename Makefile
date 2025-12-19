# Variables
GIT := git
PNPM := pnpm
CARGO := cargo
DOCKER := docker
CD := cd

NEOCRATES_PATH := ./neocrates


# Function to check if there are changes to commit
define git_push_if_needed
	@if [ -n "$$($(GIT) status --porcelain)" ]; then \
		$(GIT) add .; \
		$(GIT) commit -m "$(m)"; \
		$(GIT) push; \
	else \
		echo "No changes to commit"; \
	fi
endef

define git_commit_if_needed
	@if [ -n "$$($(GIT) status --porcelain)" ]; then \
		$(GIT) add .; \
		$(GIT) commit -m "$(m)"; \
	else \
		echo "No changes to commit"; \
	fi
endef

# Git run add commit push
git-run:
	$(call git_push_if_needed)

# Git run add commit push
git-commit:
	$(call git_commit_if_needed)

# Build facade crate
build:
	@echo "===> Build neocrates"
	$(CARGO) build -p neocrates || exit 1

# Test facade crate
test:
	@echo "===> Test neocrates"
	$(CARGO) test -p neocrates || exit 1

# neocrate watch commands
clean:
	@echo "Cleaning neocrate in $(NEOCRATES_PATH)..."
	cd $(NEOCRATES_PATH) && $(CARGO) clean
# neocrate dry run
dry-run:
	@echo "===> Dry-run neocrates"
	$(call git_commit_if_needed)
	cd $(NEOCRATES_PATH) &&  $(CARGO) publish -p neocrates --dry-run --registry crates-io || exit 1

# Publish facade crate to crates.io (requires `cargo login`)
publish:
	@echo "===> Publishing neocrates"
	$(call git_commit_if_needed)
	cd $(NEOCRATES_PATH) &&  $(CARGO) publish -p neocrates --registry crates-io || exit 1
	cd $(NEOCRATES_PATH) && $(CARGO) clean

# Publish all crates in the correct order
publish-all:
	@echo "===> Publishing Level 0 (Independent Crates)..."
	# Use '|| true' to continue if version already exists
	$(CARGO) publish -p helper --registry crates-io || true
	$(CARGO) publish -p logger --registry crates-io || true
	$(CARGO) publish -p crypto --registry crates-io || true
	$(CARGO) publish -p response --registry crates-io || true
	$(CARGO) publish -p awss3 --registry crates-io || true
	$(CARGO) publish -p awssts --registry crates-io || true
	$(CARGO) publish -p sms --registry crates-io || true
	$(CARGO) publish -p dieselhelper --registry crates-io || true

	@echo "===> Waiting 20s for registry sync..."
	@sleep 20

	@echo "===> Publishing Level 1..."
	$(CARGO) publish -p rediscache --registry crates-io || true

	@echo "===> Publishing Level 2..."
	$(CARGO) publish -p middleware --registry crates-io || true

	@echo "===> Waiting 10s for registry sync..."
	@sleep 10

	@echo "===> Publishing Final Crate (neocrates)..."
	$(CARGO) publish -p neocrates --registry crates-io
