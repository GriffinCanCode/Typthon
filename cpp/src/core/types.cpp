#include "typthon/core/types.hpp"
#include <unordered_map>
#include <vector>
#include <algorithm>

namespace typthon {

// Type hierarchy graph (adjacency list)
static std::unordered_map<TypeId, std::vector<TypeId>> subtype_graph;
static std::unordered_map<TypeId, std::vector<TypeId>> supertype_graph;

TypeId TypeLattice::meet(TypeId a, TypeId b) {
    if (a == b) return a;
    if (is_subtype(a, b)) return a;
    if (is_subtype(b, a)) return b;

    // Find common subtypes and return most specific
    // This is a simplified implementation
    // In practice, would use LCA in type hierarchy
    return 0; // Bottom type
}

TypeId TypeLattice::join(TypeId a, TypeId b) {
    if (a == b) return a;
    if (is_subtype(a, b)) return b;
    if (is_subtype(b, a)) return a;

    // Find common supertypes and return most general
    // This is a simplified implementation
    // In practice, would use LCA in reversed type hierarchy
    return 1; // Top type (Any)
}

bool TypeLattice::is_subtype(TypeId a, TypeId b) {
    if (a == b) return true;

    // BFS to check if path exists from a to b in subtype graph
    std::vector<bool> visited(4096, false);
    std::vector<TypeId> queue;
    queue.push_back(a);
    visited[a] = true;

    size_t front = 0;
    while (front < queue.size()) {
        TypeId current = queue[front++];

        auto it = supertype_graph.find(current);
        if (it != supertype_graph.end()) {
            for (TypeId super : it->second) {
                if (super == b) return true;
                if (!visited[super]) {
                    visited[super] = true;
                    queue.push_back(super);
                }
            }
        }
    }

    return false;
}

} // namespace typthon

