import { min } from 'fp-ts/lib/NonEmptyArray';
import { Edge, Graph } from '../graph';
import * as NonEmptyArray from 'fp-ts/NonEmptyArray';
import * as Option from 'fp-ts/Option';
import {
  joinNodesOnEdge,
  minimumBy,
  numberOfConnectedComponentsComponents,
} from './common';
import { createGraphUrl } from '../graphGenerator';

function evaluateEdgesComponentsCount(graph: Graph) {
  const initialComponentCount = numberOfConnectedComponentsComponents(graph);
  for (const edge of graph.edges) {
    const workingGraph = graph.shallowClone();

    joinNodesOnEdge(workingGraph, edge);

    const newComponentCount = numberOfConnectedComponentsComponents(graph);
    edge[2] = newComponentCount - initialComponentCount;
  }
  console.log(
    'Partial data, least losing component graph: ',
    createGraphUrl(graph),
  );
}

export function runLeastLosingComponents(_graph: Graph): Graph {
  const graph = _graph.shallowClone();

  while (graph.edges.length > 0) {
    evaluateEdgesComponentsCount(graph);

    var edgeToJoin = minimumBy(graph.edges, (x) => x[2]);

    joinNodesOnEdge(graph, edgeToJoin);
  }

  return graph;
}
