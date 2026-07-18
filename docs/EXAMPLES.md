# EXAMPLES.md

`nap` is a domain-agnostic entity management system. By using custom entity types and a repository-defined structure, you can adapt `nap` to store, track, and manage information in any domain.

The following examples assume a root `repository.yaml` file defining the valid entity types for the project.

---

## 1. Scientific Research Repository
**Domain:** Academic research, tracking experimental data and publication metadata.
**Types:** `paper`, `experiment`, `dataset`

### Setup
```yaml
# repository.yaml
types:
  - paper
  - experiment
  - dataset
```

### Commands
```bash
# Create a new research paper entity
nap create paper/cold-fusion-v2 -n "Replicating Cold Fusion Results"

# Assign domain-specific properties
nap set nap://lab/paper/cold-fusion-v2 status "peer-reviewed"
nap set nap://lab/paper/cold-fusion-v2 doi "10.1038/example"

# Add representation (the actual PDF)
nap add-repr nap://lab/paper/cold-fusion-v2 ./data/manuscript.pdf
```

---

## 2. IT Inventory Management
**Domain:** Tracking hardware, software licenses, and their physical locations.
**Types:** `device`, `location`, `vendor`

### Setup
```yaml
# repository.yaml
types:
  - device
  - location
  - vendor
```

### Commands
```bash
# Create a new device entity
nap create device/macbook-m4-042 -n "Engineer Laptop 042"

# Assign domain-specific properties
nap set nap://it/device/macbook-m4-042 serial_number "A123-BC456"
nap set nap://it/device/macbook-m4-042 assigned_to "jane.doe@company.com"

# Add representation (purchase receipt)
nap add-repr nap://it/device/macbook-m4-042 ./docs/receipts/mbp-042.pdf
```

---

## 3. World-Building & Lore (Entertainment)
**Domain:** A creative project developing a fantasy world, tracking deities, geography, and historical events.
**Types:** `deity`, `region`, `event`

### Setup
```yaml
# repository.yaml
types:
  - deity
  - region
  - event
```

### Commands
```bash
# Create a new deity entity
nap create deity/solaris -n "Solaris, The Sun Bringer"

# Assign domain-specific properties
nap set nap://fantasy/deity/solaris alignment "Lawful Good"
nap set nap://fantasy/deity/solaris domain "Light"

# Add representation (character concept art)
nap add-repr nap://fantasy/deity/solaris ./assets/art/solaris_concept.png
```

---

## 4. Software Architectural Documentation
**Domain:** Cataloging internal services, API endpoints, and error codes within a microservices ecosystem.
**Types:** `service`, `endpoint`, `error_code`

### Setup
```yaml
# repository.yaml
types:
  - service
  - endpoint
  - error_code
```

### Commands
```bash
# Create a new service entity
nap create service/auth-provider -n "Authentication Service"

# Assign domain-specific properties
nap set nap://platform/service/auth-provider language "Rust"
nap set nap://platform/service/auth-provider owner "platform-team"

# Add representation (architecture diagram)
nap add-repr nap://platform/service/auth-provider ./docs/diagrams/auth-flow.svg
```

***

### Summary of Workflow
Regardless of the domain, the `nap` interaction remains consistent:
1.  **Define:** Add your types to `repository.yaml`.
2.  **Create:** Use `nap create <type>/<id>` to initialize an entity.
3.  **Set:** Use `nap set <uri> <key> <value>` to attach metadata.
4.  **Represent:** Use `nap add-repr <uri> <file>` to attach files, assets, or documentation.
