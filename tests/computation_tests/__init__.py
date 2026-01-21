import json
import os
from pathlib import Path

import requests
import toml


def is_asset_none(asset):
    return asset._value is None


# TODO: THESE ARE PULLED FROM THE CLI MODULE. THEY SHOULD BE MOVED TO A SHARED MODULE.
#
# Define the path for governance data
GOVERNANCE_PATH = os.path.expanduser("~/.eqty-sdk/gov")
PROJECTS_FILE = os.path.join(GOVERNANCE_PATH, "projects.json")
FRAMEWORKS_FILE = os.path.join(GOVERNANCE_PATH, "frameworks.json")
ASSIGNMENTS_FILE_PATH = Path(GOVERNANCE_PATH) / "assignments" / "current.json"
SDK_CONFIG_PATH = Path.home() / ".eqty-sdk" / "sdk_config.toml"


def download_and_save_file(url, local_path):
    try:
        response = requests.get(url)
        response.raise_for_status()
        with open(local_path, "wb") as file:
            file.write(response.content)
    except requests.RequestException as e:
        raise Exception(f"Failed to download or save file: {e}")


def sync_projects_with_assignments(assignments_data, sdk_config):
    """Synchronizes the projects from the SDK config with the assignments data."""
    projects_in_assignments = {assignment["projectId"] for assignment in assignments_data}
    for project_name in sdk_config.get("projects", {}):
        if project_name not in projects_in_assignments:
            assignments_data.append(
                {"projectId": project_name, "frameworkId": "", "complianceRecords": []}
            )


def load_framework_controls(framework_id):
    """Loads the controls for a given framework."""
    controls_file_path = Path(GOVERNANCE_PATH) / "frameworks" / framework_id / "controls.json"
    if controls_file_path.exists():
        with open(controls_file_path, "r") as file:
            controls = json.load(file)
            return controls
    else:
        return []  # todo: handle error


def setup_framework(framework_id_param: str) -> bool:
    framework_id_param = framework_id_param.lower()

    frameworks_url = (
        "https://huggingface.co/datasets/open-responsibility/frameworks/raw/main/catalog.json"
    )
    local_frameworks_path = Path.home() / ".eqty-sdk" / "gov" / "frameworks" / framework_id_param
    framework_json_filename = "framework.json"
    controls_json_filename = "controls.json"

    try:
        response = requests.get(frameworks_url)
        response.raise_for_status()
        frameworks = response.json()

        framework_data = next(
            (f for f in frameworks if f["frameworkId"] == framework_id_param), None
        )
        if not framework_data:
            print(f"Framework ID '{framework_id_param}' not found in remote repository.")
            return False

        # Create local directory if it doesn't exist
        local_frameworks_path.mkdir(parents=True, exist_ok=True)

        # Check for existing framework and handle the update case
        local_framework_file = local_frameworks_path / framework_json_filename
        local_controls_file = local_frameworks_path / controls_json_filename
        if local_framework_file.exists() or local_controls_file.exists():
            print("Framework already exists. Use '--update' to update the local version.")
            return True
        else:
            print("Adding new framework to the local environment...")

        # Download and save framework.json
        framework_json_url = f"https://huggingface.co/datasets/open-responsibility/frameworks/raw/main/{framework_data['path']}/{framework_json_filename}"
        download_and_save_file(framework_json_url, local_framework_file)

        # Download and save controls.json
        controls_json_url = f"https://huggingface.co/datasets/open-responsibility/frameworks/raw/main/{framework_data['path']}/{controls_json_filename}"
        download_and_save_file(controls_json_url, local_controls_file)

        print(f"Framework '{framework_id_param}' added/updated successfully.")

    except requests.RequestException as e:
        print(f"Error fetching frameworks data: {e}")
        return False

    return True


def setup_governance(framework_id: str, project_id: str):
    local_frameworks_base_path = Path.home() / ".eqty" / "gov" / "frameworks"
    local_framework_path = local_frameworks_base_path / framework_id

    success = setup_framework(framework_id)

    if not success:
        print("Failed to setup governance framework.")
        return

    # Check if the framework is installed locally
    if not local_framework_path.exists():
        print(
            f"Framework '{framework_id}' is not installed. Use 'cli gov add {framework_id}' to install it."
        )
        return

    # Load SDK config data
    if not SDK_CONFIG_PATH.exists():
        print("SDK config file not found.")
        return

    with open(SDK_CONFIG_PATH, "r") as file:
        sdk_config = toml.load(file)

    # Check if the project exists
    if project_id not in sdk_config.get("projects", {}):
        print(f"Project '{project_id}' not found in SDK config.")
        return

    # Load assignments data
    if not ASSIGNMENTS_FILE_PATH.exists():
        print("Assignments file not found. Creating a new one.")
        assignments_data = []
    else:
        with open(ASSIGNMENTS_FILE_PATH, "r") as file:
            assignments_data = json.load(file)

    # Check if the framework is already assigned to the project
    for assignment in assignments_data:
        if assignment["projectId"] == project_id and framework_id in assignment.get(
            "frameworkId", ""
        ):
            print(f"Framework '{framework_id}' is already assigned to project '{project_id}'.")
            return

    # Load controls for the framework
    controls = load_framework_controls(framework_id)

    # Initialize compliance records
    compliance_records = [
        {
            "controlId": control["controlId"],
            "status": "not-started",
            "comments": [],
            "evidenceUrls": [],
        }
        for control in controls
    ]

    # Assign the framework to the project
    assignments_data.append(
        {
            "projectId": project_id,
            "frameworkId": framework_id,
            "complianceRecords": compliance_records,
        }
    )

    # Ensure ASSIGNMENTS_FILE_PATH exists, create if not
    ASSIGNMENTS_FILE_PATH.parent.mkdir(parents=True, exist_ok=True)
    if not ASSIGNMENTS_FILE_PATH.exists():
        ASSIGNMENTS_FILE_PATH.write_text(json.dumps([]))

    # Save updated assignments data back to file
    with open(ASSIGNMENTS_FILE_PATH, "w") as file:
        json.dump(assignments_data, file, indent=4)
