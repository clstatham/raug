export type GraphElementGroup = 'nodes' | 'edges';

export interface GraphElement {
	group: GraphElementGroup;
	data: {
		id: string;
		label?: string;
		[key: string]: unknown;
	};
	position?: {
		x: number;
		y: number;
	};
}

export type GraphElements = GraphElement | GraphElement[];

const ensureArray = (elements: GraphElements): GraphElement[] => {
	return Array.isArray(elements) ? elements : [elements];
};

export default class Editor {
	readonly container: HTMLElement;

	readonly cy: {
		add: (elements: GraphElements) => void;
	};

	constructor(container: HTMLElement) {
		this.container = container;
		this.container.classList.add('graph-editor');
		if (!this.container.style.position) {
			this.container.style.position = 'relative';
		}
		this.cy = {
			add: (elements: GraphElements) => {
				ensureArray(elements).forEach((element) => this.renderElement(element));
			},
		};
	}

	private renderElement(element: GraphElement): void {
		const elementNode = document.createElement('div');
		elementNode.className = `graph-element graph-element-${element.group}`;
		elementNode.textContent = element.data.label ?? element.data.id;
		elementNode.style.position = 'absolute';

		if (element.position) {
			elementNode.style.left = `${element.position.x}px`;
			elementNode.style.top = `${element.position.y}px`;
		}

		this.container.appendChild(elementNode);
	}
}
