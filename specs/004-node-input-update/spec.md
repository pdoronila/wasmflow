# Feature Specification: Four-Section Node Layout

**Feature Branch**: `004-node-input-update`
**Created**: 2025-10-16
**Status**: Draft
**Input**: User description: "node-input-update current a node has 3 main sections, the header, body and footer. This has been working well but soon I will have more complex nodes with multiple fields. Im thinking lets update the number of sections to four. the header, the connections (for input and output), the body that can be customized by the component like how custom footer pattern, and last the footer."

**Additional Clarifications**:
- "The Footer default view should be the dynamic status information. If a component provides a custom footer view, than the footer should be what the component provides."
- "The footer has a default view and custom view, the body should also support this. There is the default body view and is replaced if a component provides their own body view"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Node Connections Separately (Priority: P1)

Users need to see input and output connections in a dedicated section separate from the header and body, enabling them to quickly understand data flow without parsing mixed content.

**Why this priority**: This is the foundation of the new layout structure. Without a dedicated connections section, the entire four-section architecture cannot function. It enables users to distinguish between node identity (header), data flow (connections), interaction (body), and status (footer).

**Independent Test**: Can be fully tested by creating a node with multiple input/output connections and verifying that they appear in a distinct section between the header and body. Delivers immediate value by improving visual separation of node concerns.

**Acceptance Scenarios**:

1. **Given** a node with 2 inputs (value:u32, freq:u32) and 1 output (value:u32), **When** the user views the node, **Then** the connections section displays all inputs on the left with "o" pins and outputs on the right with "o" pins
2. **Given** a node with no connections, **When** the user views the node, **Then** the connections section is still present but appears empty or collapsed
3. **Given** a node with input-only connections, **When** the user views the node, **Then** the connections section shows inputs on the left side with no outputs on the right

---

### User Story 2 - Display Default and Custom Body Content (Priority: P2)

Users need to see default interactive content in the body section (such as input fields for all node parameters), but components should be able to override this with custom body content when they need specialized controls or layouts.

**Why this priority**: Once the connections section exists (P1), users need a place to interact with node parameters. Providing a default body view ensures all nodes are immediately usable, while allowing customization supports advanced use cases.

**Independent Test**: Can be tested by (1) creating a node without custom body content and verifying it displays default input fields for all parameters, and (2) creating a node with custom body content and verifying the custom content overrides the default display.

**Acceptance Scenarios**:

1. **Given** a node with parameters (value:u32, freq:u32) and no custom body, **When** the user views the node, **Then** the body section displays default input fields for "value" and "freq" between the connections and footer sections
2. **Given** a component provides custom body content (e.g., specialized controls), **When** the user views the node, **Then** the body section displays the custom content instead of the default input fields
3. **Given** a user edits an input field in the default body section, **When** the value changes, **Then** the footer section updates to reflect the new value (if footer displays current values)
4. **Given** a node with no parameters and no custom body, **When** the user views the node, **Then** the body section is empty or shows placeholder text

---

### User Story 3 - Display Default and Custom Footer Content (Priority: P3)

Users need to see dynamic status information in the footer by default (such as current parameter values or computation results), but components should be able to override this with custom footer content when they need specialized displays.

**Why this priority**: This enhances the usability of the four-section layout but is not essential for the layout to function. It depends on P1 (layout structure) and P2 (body customization) existing first. The default status display provides immediate value while allowing future customization.

**Independent Test**: Can be tested by (1) creating a node without custom footer content and verifying it displays default dynamic status values, and (2) creating a node with custom footer content and verifying the custom content overrides the default display.

**Acceptance Scenarios**:

1. **Given** a node with current state values (value=100, freq=50) and no custom footer, **When** the user views the node, **Then** the footer section displays default dynamic status: "Current value: 100" and "Current freq: 50"
2. **Given** a node's internal state changes, **When** the computation completes, **Then** the default footer updates to show the new values
3. **Given** a component provides custom footer content, **When** the user views the node, **Then** the footer section displays the custom content instead of the default dynamic status
4. **Given** a node with no dynamic state and no custom footer, **When** the user views the node, **Then** the footer section is empty or shows placeholder text

---

### Edge Cases

- What happens when a node has more than 10 input/output connections (does the connections section scroll, collapse, or expand)?
- How does the system handle nodes with very long parameter names in the connections section (truncation, wrapping, tooltips)?
- What happens when custom body content exceeds available vertical space?
- How does the layout adapt when a node is resized or collapsed?
- What happens when a component tries to customize the header or connections section (are these sections locked or customizable)?
- What happens when a component provides custom body content but later removes it (does it revert to default input fields)?
- What happens when a component provides custom footer content but later removes it (does it revert to default dynamic status)?
- How does the system determine which parameters to show input fields for in the default body when a node has many parameters?
- How does the system determine which parameter values to display in the default footer when a node has many parameters?
- What happens when a parameter type doesn't have a standard input field widget (e.g., complex nested types)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST render nodes with four distinct visual sections: header, connections, body, and footer (in that order from top to bottom)
- **FR-002**: System MUST display the node name/title in the header section
- **FR-003**: System MUST display all input connections on the left side of the connections section with visual pin indicators
- **FR-004**: System MUST display all output connections on the right side of the connections section with visual pin indicators
- **FR-005**: System MUST display connection type information (e.g., "value:u32") alongside each connection pin
- **FR-006**: System MUST display default interactive content (input fields for node parameters) in the body section when a component does not provide custom body content
- **FR-007**: System MUST allow components to define custom content for the body section that overrides the default interactive content
- **FR-008**: System MUST display component-provided custom body content when available, taking precedence over default input fields
- **FR-009**: System MUST render body content (default or custom) between the connections and footer sections
- **FR-010**: System MUST display default dynamic status information in the footer section when a component does not provide custom footer content
- **FR-011**: System MUST allow components to define custom content for the footer section that overrides the default dynamic status display
- **FR-012**: System MUST display component-provided custom footer content when available, taking precedence over default dynamic status
- **FR-013**: System MUST maintain visual separation between all four sections using borders or spacing
- **FR-014**: System MUST preserve existing node functionality (connection behavior, data flow) when migrating from three-section to four-section layout
- **FR-015**: System MUST support nodes with zero, one, or multiple input/output connections in the connections section

### Key Entities

- **Node**: A visual element in the node editor with four sections (header, connections, body, footer) that represents a computation or component
- **Connection**: An input or output data path with a type (e.g., u32) and optional label (e.g., "value", "freq") displayed in the connections section
- **Pin**: A visual indicator on the left (input) or right (output) of the connections section that shows where connections can be made
- **Section**: A distinct visual area within a node (header, connections, body, or footer) with specific content responsibilities

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can identify all input and output connections by looking only at the connections section (100% accuracy in user testing)
- **SC-002**: Users can distinguish between connection information, interactive controls, and status displays without confusion (90% task completion rate in usability testing)
- **SC-003**: Nodes with multiple fields render all four sections in the correct order (header → connections → body → footer) consistently
- **SC-004**: Existing nodes migrated from three-section layout maintain all functionality without data loss or connection breaks (100% backward compatibility)
- **SC-005**: Components can customize body and footer content independently without affecting header or connections sections (verified through component API testing)
- **SC-006**: Visual hierarchy is clear with users able to identify the purpose of each section within 5 seconds of viewing a node
- **SC-007**: Nodes without custom body content display default input fields for all parameters immediately upon creation (100% of nodes show interactive controls by default)

## Assumptions

- The current three-section layout (header, body, footer) is used across all existing nodes
- Components already have a mechanism to customize the footer section (mentioned as "custom footer pattern")
- The visual design uses borders or spacing to separate sections
- Connection pins are currently displayed but may be mixed with other content in the body section
- The migration from three to four sections should preserve existing component customization patterns
- Input fields in the body section will use standard UI controls available in the node editor framework
- The body section will display default input fields for all node parameters by default, with components able to override with custom content
- Default body content includes interactive input fields for each parameter (e.g., "value" input field, "freq" input field)
- When a component provides custom body content, the entire body is replaced (not merged with default input fields)
- The footer section will primarily display read-only status information by default, with components able to override with custom content
- Default dynamic status information includes displaying current values of node parameters (e.g., "Current value: X")
- When a component provides custom footer content, the entire footer is replaced (not merged with default status)
