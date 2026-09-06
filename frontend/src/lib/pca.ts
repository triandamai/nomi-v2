/**
 * Minimal multi-component PCA for plotting high-dimensional embeddings on a low-dimensional
 * chart. No dependency needed: each principal component is found by power iteration on the
 * covariance matrix, deflating out prior components before finding the next one. This is
 * standard for a "top few components" case and avoids pulling in a full linear algebra library
 * for something this small.
 */

function dot(a: number[], b: number[]): number {
	let sum = 0;
	for (let i = 0; i < a.length; i++) sum += a[i] * b[i];
	return sum;
}

function norm(a: number[]): number {
	return Math.sqrt(dot(a, a));
}

function scale(a: number[], s: number): number[] {
	return a.map((x) => x * s);
}

function subtract(a: number[], b: number[]): number[] {
	return a.map((x, i) => x - b[i]);
}

/** One power-iteration pass to find the dominant eigenvector of `matrix` (rows = samples). */
function dominantEigenvector(matrix: number[][], dims: number, iterations = 100): number[] {
	let vector = Array.from({ length: dims }, () => Math.random() - 0.5);
	for (let iter = 0; iter < iterations; iter++) {
		// next = Mᵀ(M v) — avoids materializing the full dims×dims covariance matrix.
		const mv = matrix.map((row) => dot(row, vector));
		const next = new Array(dims).fill(0);
		for (let i = 0; i < matrix.length; i++) {
			for (let j = 0; j < dims; j++) {
				next[j] += matrix[i][j] * mv[i];
			}
		}
		const magnitude = norm(next);
		if (magnitude === 0) return vector;
		vector = scale(next, 1 / magnitude);
	}
	return vector;
}

/** Projects each row of `vectors` onto its top `numComponents` principal components. */
function projectToComponents(vectors: number[][], numComponents: number): number[][] {
	if (vectors.length < 2) return vectors.map(() => new Array(numComponents).fill(0));

	const dims = vectors[0].length;
	const mean = new Array(dims).fill(0);
	for (const v of vectors) {
		for (let j = 0; j < dims; j++) mean[j] += v[j] / vectors.length;
	}
	const centered = vectors.map((v) => subtract(v, mean));

	const components: number[][] = [];
	let deflated = centered;
	for (let c = 0; c < numComponents; c++) {
		const pc = dominantEigenvector(deflated, dims);
		components.push(pc);
		// Deflate: remove each row's projection onto this component before finding the next one,
		// so it captures the next-largest independent direction of variance instead of re-finding
		// the same one.
		deflated = deflated.map((row) => subtract(row, scale(pc, dot(row, pc))));
	}

	return centered.map((row) => components.map((pc) => dot(row, pc)));
}

/** Projects each row of `vectors` onto 3D via PCA. Returns the origin for fewer than 2 rows. */
export function projectTo3D(vectors: number[][]): { x: number; y: number; z: number }[] {
	return projectToComponents(vectors, 3).map(([x, y, z]) => ({ x, y, z }));
}
