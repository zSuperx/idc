#![allow(unused)]
/// This is "forked"/copied from heuristic-graph-coloring v0.1.0
/// Credits go to: victorvde
/// https://docs.rs/heuristic-graph-coloring/latest/heuristic_graph_coloring/index.html
use std::slice::Iter;

/// Trait for graphs that the algorithms in this crate can work with.
pub trait ColorableGraph {
    /// Total number of vertices in the graph.
    fn num_vertices(&self) -> usize;
    /// All neighbors of vertex `vi`
    fn neighbors(&self, vi: usize) -> &[usize];

    /// Degree of vertex `vi`.
    fn degree(&self, vi: usize) -> usize {
        self.neighbors(vi).len()
    }

    /// Maximum degree of the graph.
    fn max_degree(&self) -> usize {
        (0..self.num_vertices())
            .map(|i| self.degree(i))
            .max()
            .unwrap_or(0)
    }
}

/// Graph represented as multiple `Vec`s, one for each vertex.
#[derive(Debug, Clone)]
pub struct VecVecGraph {
    edges: Vec<Vec<usize>>,
}

impl ColorableGraph for &VecVecGraph {
    fn num_vertices(&self) -> usize {
        self.edges.len()
    }

    fn neighbors(&self, vi: usize) -> &[usize] {
        &self.edges[vi]
    }
}

impl VecVecGraph {
    /// Create graph with `num_vertices` vertices and no edges.
    pub fn new(num_vertices: usize) -> Self {
        VecVecGraph {
            edges: (0..num_vertices).map(|_| vec![]).collect(),
        }
    }

    pub fn iter(&self) -> Iter<'_, Vec<usize>> {
        self.edges.iter()
    }

    /// Add an edge between `w` and `v`.
    ///
    /// Note: avoid adding edges multiple times or adding an edge of a vertex to itself (a loop) as this confuses the algorithms.
    pub fn add_edge(&mut self, w: usize, v: usize) {
        self.edges[w].push(v);
        self.edges[v].push(w);
    }

    /// Load a graph from a DIMACS formatted text file (`*.col`).
    ///
    /// Note that DIMACS files start vertex indices at 1 while this library starts vertex indices at 0.
    pub fn from_dimacs_file(
        path: &dyn AsRef<std::path::Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use std::io::BufRead;

        let f = std::fs::File::open(path)?;
        let r = std::io::BufReader::new(f);
        let mut graph = None;
        let rgraph = &mut graph;
        for (i, line) in r.lines().enumerate() {
            let line = line?;
            (|| {
                let mut it = line.split(' ');
                match it.next() {
                    Some("c" | "n" | "x" | "d" | "v") | None => {}
                    Some("p") => {
                        // problem statement with number of ertices
                        let format = it.next()?;
                        if format != "edge" {
                            return None;
                        }
                        let num_vertices = it.next()?.parse::<usize>().ok()?;
                        let _num_edges = it.next()?.parse::<usize>().ok()?;
                        if it.next().is_some() {
                            return None;
                        }
                        if rgraph.is_some() {
                            return None;
                        }
                        *rgraph = Some(Self::new(num_vertices));
                    }
                    Some("e") => {
                        // edge
                        let w = it.next()?.parse::<usize>().ok()? - 1;
                        let v = it.next()?.parse::<usize>().ok()? - 1;
                        let max = rgraph.as_ref()?.num_vertices();
                        if w >= max || v >= max {
                            return None;
                        }
                        if w != v {
                            rgraph.as_mut()?.add_edge(w, v);
                        }
                    }
                    _ => return None,
                }
                Some(())
            })()
            .ok_or(format!("invalid line {}: {}", i + 1, line))?;
        }
        Ok(graph.ok_or("no graph defined in file")?)
    }
}

/// A graph in compressed format with all edges in one array.
///
/// Very slightly faster than [VecVecGraph] (excluding the creation time).
#[derive(Debug, Clone)]
pub struct CsrGraph {
    vertices: Vec<usize>,
    edges: Vec<usize>,
}

impl ColorableGraph for &CsrGraph {
    fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    fn neighbors(&self, vi: usize) -> &[usize] {
        &self.edges[if vi == 0 { 0 } else { self.vertices[vi - 1] }..self.vertices[vi]]
    }
}

impl<T> From<T> for CsrGraph
where
    T: ColorableGraph,
{
    fn from(graph: T) -> Self {
        let mut vertices = Vec::with_capacity(graph.num_vertices());
        let mut edges = vec![];
        let mut offset = 0;
        for i in 0..graph.num_vertices() {
            let neighbors = graph.neighbors(i);
            edges.extend(neighbors);
            offset += neighbors.len();
            vertices.push(offset);
        }
        CsrGraph { vertices, edges }
    }
}

fn color_greedy_by_order(
    graph: impl ColorableGraph,
    order: impl Iterator<Item = usize>,
    coloring: Option<Vec<usize>>,
) -> Vec<usize> {
    let mut coloring = coloring.unwrap_or(vec![usize::MAX; graph.num_vertices()]);

    for i in order {
        let mut ncs = vec![];
        for &n in graph.neighbors(i) {
            let nc = coloring[n];
            if nc != usize::MAX {
                if nc >= ncs.len() {
                    ncs.resize(nc + 1, false);
                }
                ncs[nc] = true;
            }
        }
        for c in 0.. {
            if c >= ncs.len() || !ncs[c] {
                coloring[i] = c;
                break;
            }
        }
    }

    coloring
}

/// The most simple greedy algorigthm. Colors vertices in index order by first possible color.
///
/// Use as baseline only, [color_greedy_by_degree] generally uses less colors in the same time.
pub fn color_greedy_naive(graph: impl ColorableGraph) -> Vec<usize> {
    let range = 0..graph.num_vertices();
    color_greedy_by_order(graph, range, None)
}

/// Colors vertices from highest degree to lowest by first possible color.
///
/// Generally uses less colors than [color_greedy_naive] in about the same time.
/// Generally uses more colors than [color_greedy_dsatur] and [color_rlf], but in less time.
pub fn color_greedy_by_degree(graph: impl ColorableGraph) -> Vec<usize> {
    let mut vertices: Vec<_> = (0..graph.num_vertices()).collect();
    vertices.sort_by_key(|&i| std::cmp::Reverse(graph.neighbors(i).len()));
    color_greedy_by_order(graph, vertices.iter().cloned(), None)
}

/// Same as `color_greedy_by_degree()` except allows you to precolor nodes
pub fn color_greedy_by_degree_with_precolor(
    graph: impl ColorableGraph,
    coloring: Vec<usize>,
) -> Vec<usize> {
    let mut vertices: Vec<_> = (0..graph.num_vertices()).collect();
    vertices.sort_by_key(|&i| std::cmp::Reverse(graph.neighbors(i).len()));
    color_greedy_by_order(graph, vertices.iter().cloned(), Some(coloring))
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
struct DSatur {
    saturation: usize,
    degree_uncolored: usize,
    index: std::cmp::Reverse<usize>,
}

/// Colors vertices from most colors of neighbors to least (dynamically) by first possible color. Known as DSATUR.
///
/// Generally uses less colors than [color_greedy_by_degree], but in more time.
/// Generally uses more colors than [color_rlf], but in less time.
pub fn color_greedy_dsatur(graph: impl ColorableGraph) -> Vec<usize> {
    let mut coloring = vec![usize::MAX; graph.num_vertices()];

    // to control the time complexity we don't go thorough all the neighbours of neighbours every time we color
    let mut neighbor_coloring: Vec<Vec<bool>> = vec![];
    neighbor_coloring.resize_with(graph.num_vertices(), Vec::new);

    // initialize priority queue / heap
    let mut dsaturs = vec![];
    for i in 0..graph.num_vertices() {
        dsaturs.push(DSatur {
            saturation: 0,
            degree_uncolored: graph.neighbors(i).len(),
            index: std::cmp::Reverse(i),
        });
    }
    let mut heap = std::collections::BTreeSet::from_iter(dsaturs.iter().cloned());

    loop {
        // get most saturated vertex
        // TODO: use pop_last when stabilized
        let dsatur = match heap.iter().last() {
            None => break,
            Some(x) => x.clone(),
        };
        let i = dsatur.index.0;
        let removed = heap.remove(&dsatur);
        assert!(removed);

        // color it
        let ncs = &neighbor_coloring[i];
        let mut c = usize::MAX;
        for ci in 0.. {
            if ci >= ncs.len() || !ncs[ci] {
                c = ci;
                break;
            }
        }
        assert!(coloring[i] == usize::MAX);
        coloring[i] = c;

        // update the counts of all neighbors
        for &n in graph.neighbors(i) {
            // get neighbor
            if coloring[n] != usize::MAX {
                continue;
            }
            let dsatur = &mut dsaturs[n];
            let removed = heap.remove(dsatur);
            assert!(removed);

            // remove color from neighbor and change counts
            let ncs = &mut neighbor_coloring[n];
            if c >= ncs.len() {
                ncs.resize(c + 1, false);
            }
            let has_color = ncs[c];
            if !has_color {
                ncs[c] = true;
                dsatur.saturation += 1;
            }
            dsatur.degree_uncolored -= 1;

            heap.insert(dsatur.clone());
        }
    }

    coloring
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Rlf {
    next_neighbors: usize,
    other_neighbors: std::cmp::Reverse<usize>,
    index: std::cmp::Reverse<usize>,
}

/// Colors vertices one color at a time by foming maximal independent sets. Known as Recursive Largest First.
///
/// Generally uses less colors than [color_greedy_by_degree] and [color_greedy_dsatur], but in more time.
pub fn color_rlf(graph: impl ColorableGraph) -> Vec<usize> {
    let mut coloring = vec![usize::MAX; graph.num_vertices()];
    let mut degree_next: Vec<usize> = vec![0; graph.num_vertices()];
    let mut degree_other: Vec<usize> = vec![0; graph.num_vertices()];

    let mut nocolor = usize::MAX;
    let mut nextcolor = usize::MAX - 1;

    for c in 0.. {
        // reinitialize degrees
        for i in 0..graph.num_vertices() {
            if coloring[i] != nocolor {
                continue;
            }

            degree_next[i] = 0;
            degree_other[i] = graph
                .neighbors(i)
                .iter()
                .filter(|&&n| coloring[n] == nocolor)
                .count();
        }

        // get first vertex with maximum degree
        let mut current_vertex = (0..graph.num_vertices())
            .filter(|&i| coloring[i] == nocolor)
            .max_by_key(|&i| (degree_other[i], std::cmp::Reverse(i)));
        if current_vertex.is_none() {
            break;
        }

        while let Some(i) = current_vertex {
            debug_assert!(coloring[i] == nocolor);
            coloring[i] = c;

            // color direct neighbours
            for &n in graph.neighbors(i) {
                if coloring[n] != nocolor {
                    continue;
                }
                coloring[n] = nextcolor;

                // change degrees for indirect neighbors
                for &n2 in graph.neighbors(n) {
                    if coloring[n2] != nocolor {
                        continue;
                    }

                    degree_next[n2] += 1;
                    degree_other[n2] -= 1;
                }
            }

            // next vertex is the one with the most shared neighbors
            // TODO: use pop_last when stabilized?
            current_vertex = (0..graph.num_vertices())
                .filter(|&i| coloring[i] == nocolor)
                .max_by_key(|&i| {
                    (
                        degree_next[i],
                        std::cmp::Reverse(degree_other[i]),
                        std::cmp::Reverse(i),
                    )
                });
        }
        std::mem::swap(&mut nextcolor, &mut nocolor);
    }

    coloring
}

/// Counts the amount of colors in `coloring` by getting the maximum color plus one.
pub fn count_colors(coloring: &[usize]) -> usize {
    match coloring.iter().max() {
        None => 0,
        Some(x) => x + 1,
    }
}

/// Checks that the coloring is correct, else panics.
pub fn validate_coloring(graph: impl ColorableGraph, coloring: &[usize]) {
    for i in 0..graph.num_vertices() {
        let c = coloring[i];
        assert!(c != usize::MAX, "no color for vertex {i}");
        for &n in graph.neighbors(i) {
            assert!(
                c != coloring[n],
                "vertex {} and neighbor {} both have color {}",
                i + 1,
                n + 1,
                c
            );
        }
    }
}

/// Adjust coloring to try to make each color used a similar amount
pub fn make_coloring_more_equitable(
    graph: impl ColorableGraph,
    coloring: &mut [usize],
    count_colors: usize,
) {
    // get initial amounts
    let mut amounts = vec![0; count_colors];
    for &color in coloring.iter() {
        amounts[color] += 1;
    }

    // loop until equilibrium
    loop {
        let mut changed = false;

        for i in 0..graph.num_vertices() {
            let ci = coloring[i];
            // find available color with the least amount
            let smallest_c = match (0..count_colors)
                .filter(|&c| graph.neighbors(i).iter().all(|&j| coloring[j] != c))
                .min_by_key(|&c| amounts[c])
            {
                None => continue,
                Some(x) => x,
            };
            // change color if it would help
            if amounts[smallest_c] < amounts[ci] - 1 {
                coloring[i] = smallest_c;
                amounts[ci] -= 1;
                amounts[smallest_c] += 1;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
}
