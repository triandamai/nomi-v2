<script lang="ts">
	import * as THREE from 'three';
	import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
	import { projectTo3D } from '$lib/pca';
	import type { MemoryItem } from '$lib/types';

	let { memories }: { memories: MemoryItem[] } = $props();

	let container: HTMLDivElement;
	let selected: MemoryItem | null = $state(null);

	// Spreads the raw PCA output (arbitrary real-valued) into a fixed-size cube so the graph
	// looks reasonable regardless of how spread out or clustered the embeddings happen to be.
	const SPREAD = 4;

	function readCssColor(variable: string, fallback: number): number {
		if (typeof window === 'undefined') return fallback;
		const value = getComputedStyle(document.documentElement).getPropertyValue(variable).trim();
		if (!value) return fallback;
		const probe = document.createElement('div');
		probe.style.color = value;
		document.body.appendChild(probe);
		const rgb = getComputedStyle(probe).color;
		document.body.removeChild(probe);
		const match = rgb.match(/\d+/g);
		if (!match) return fallback;
		const [r, g, b] = match.map(Number);
		return (r << 16) + (g << 8) + b;
	}

	$effect(() => {
		if (!container || memories.length === 0) return;

		const width = container.clientWidth;
		const height = container.clientHeight;

		const scene = new THREE.Scene();
		const camera = new THREE.PerspectiveCamera(50, width / height, 0.1, 100);
		camera.position.set(8, 8, 8);

		const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
		renderer.setSize(width, height);
		renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
		container.appendChild(renderer.domElement);

		const controls = new OrbitControls(camera, renderer.domElement);
		controls.enableDamping = true;

		scene.add(new THREE.AmbientLight(0xffffff, 0.8));
		const point = new THREE.PointLight(0xffffff, 0.6);
		point.position.set(10, 10, 10);
		scene.add(point);

		const primaryColor = readCssColor('--md-sys-color-primary', 0x6750a4);

		const raw = projectTo3D(memories.map((m) => m.embedding));
		const maxAbs = Math.max(1e-6, ...raw.flatMap((p) => [Math.abs(p.x), Math.abs(p.y), Math.abs(p.z)]));

		const meshes: { mesh: THREE.Mesh; memory: MemoryItem }[] = [];
		raw.forEach((p, i) => {
			const memory = memories[i];
			const radius = 0.15 + memory.weight * 0.12;
			const geometry = new THREE.SphereGeometry(radius, 24, 24);
			const material = new THREE.MeshStandardMaterial({ color: primaryColor });
			const mesh = new THREE.Mesh(geometry, material);
			mesh.position.set((p.x / maxAbs) * SPREAD, (p.y / maxAbs) * SPREAD, (p.z / maxAbs) * SPREAD);
			scene.add(mesh);
			meshes.push({ mesh, memory });
		});

		const raycaster = new THREE.Raycaster();
		const pointer = new THREE.Vector2();

		function onClick(event: MouseEvent) {
			const rect = renderer.domElement.getBoundingClientRect();
			pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
			pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
			raycaster.setFromCamera(pointer, camera);
			const hit = raycaster.intersectObjects(meshes.map((m) => m.mesh))[0];
			if (hit) {
				selected = meshes.find((m) => m.mesh === hit.object)?.memory ?? null;
			}
		}
		renderer.domElement.addEventListener('click', onClick);

		let frameId: number;
		function animate() {
			frameId = requestAnimationFrame(animate);
			controls.update();
			renderer.render(scene, camera);
		}
		animate();

		function onResize() {
			const w = container.clientWidth;
			const h = container.clientHeight;
			camera.aspect = w / h;
			camera.updateProjectionMatrix();
			renderer.setSize(w, h);
		}
		window.addEventListener('resize', onResize);

		return () => {
			cancelAnimationFrame(frameId);
			window.removeEventListener('resize', onResize);
			renderer.domElement.removeEventListener('click', onClick);
			controls.dispose();
			meshes.forEach(({ mesh }) => {
				mesh.geometry.dispose();
				(mesh.material as THREE.Material).dispose();
			});
			renderer.dispose();
			container.removeChild(renderer.domElement);
		};
	});
</script>

<div class="flex flex-col gap-3 md:flex-row">
	<div
		bind:this={container}
		class="h-[320px] w-full rounded-lg md:h-[360px]"
		style="background: var(--md-sys-color-surface-container-low); cursor: grab"
	></div>
	<div
		class="w-full shrink-0 rounded-lg p-4 md:w-64"
		style="background: var(--md-sys-color-surface-container-low)"
	>
		{#if selected}
			<p class="md-body-medium" style="color: var(--md-sys-color-on-surface)">{selected.content}</p>
			<p class="md-body-small mt-2" style="color: var(--md-sys-color-on-surface-variant)">
				Weight {selected.weight.toFixed(2)}
			</p>
		{:else}
			<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
				Drag to rotate, scroll to zoom, click a node to see its memory and weight.
			</p>
		{/if}
	</div>
</div>
