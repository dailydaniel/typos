<script lang="ts">
  import { onMount } from "svelte";
  import { Network } from "vis-network";
  import { DataSet } from "vis-data";

  interface GraphNode {
    id: string;
    label: string;
    node_type: string;
  }

  interface GraphEdge {
    source: string;
    target: string;
  }

  interface Props {
    nodes: GraphNode[];
    edges: GraphEdge[];
    onNavigate: (id: string) => void;
  }

  let { nodes, edges, onNavigate }: Props = $props();

  let container: HTMLDivElement;

  const typeColors: Record<string, string> = {
    note: "#3b82f6",
    task: "#f59e0b",
    tag: "#8b5cf6",
    journal: "#10b981",
    report: "#ef4444",
  };

  onMount(() => {
    const visNodes = new DataSet(
      nodes.map((n) => ({
        id: n.id,
        label: n.label,
        color: {
          background: typeColors[n.node_type] ?? "#6b7280",
          border: typeColors[n.node_type] ?? "#6b7280",
          highlight: { background: "#2563eb", border: "#1d4ed8" },
        },
        font: { color: "#1f2937", size: 12 },
        shape: n.node_type === "tag" ? "diamond" : "dot",
        size: n.node_type === "tag" ? 12 : 16,
      }))
    );

    const visEdges = new DataSet(
      edges.map((e, i) => ({
        id: i,
        from: e.source,
        to: e.target,
        arrows: "to",
        color: { color: "#d1d5db", highlight: "#2563eb" },
      }))
    );

    const network = new Network(
      container,
      { nodes: visNodes, edges: visEdges },
      {
        physics: {
          solver: "forceAtlas2Based",
          forceAtlas2Based: { gravitationalConstant: -30, springLength: 120 },
          stabilization: { iterations: 150 },
        },
        interaction: {
          hover: true,
          tooltipDelay: 200,
        },
        edges: {
          smooth: { type: "continuous" },
        },
      }
    );

    network.on("doubleClick", (params: { nodes: string[] }) => {
      if (params.nodes.length > 0) {
        onNavigate(params.nodes[0]);
      }
    });

    return () => network.destroy();
  });
</script>

<div class="graph-container" bind:this={container}></div>

<style>
  .graph-container {
    width: 100%;
    height: 100%;
  }
</style>
