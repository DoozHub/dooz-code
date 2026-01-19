//! Step Ordering
//!
//! Orders steps by dependencies using topological sort.

use crate::types::{Step, StepId, ContextError};
use std::collections::{HashMap, HashSet};

/// Step orderer
pub struct StepOrderer {
    /// Enable cycle detection
    detect_cycles: bool,
}

impl StepOrderer {
    /// Create new orderer
    pub fn new() -> Self {
        Self { detect_cycles: true }
    }

    /// Order steps by dependencies
    pub fn order(&self, steps: Vec<Step>) -> Result<Vec<Step>, ContextError> {
        if steps.is_empty() {
            return Ok(steps);
        }

        // Build dependency map
        let mut id_to_step: HashMap<String, Step> = HashMap::new();
        let mut id_to_deps: HashMap<String, Vec<String>> = HashMap::new();

        for step in steps {
            let id = step.id.as_str().to_string();
            let deps: Vec<String> = step.depends_on.iter().map(|d| d.as_str().to_string()).collect();
            id_to_deps.insert(id.clone(), deps);
            id_to_step.insert(id, step);
        }

        // Topological sort
        let sorted_ids = self.topological_sort(&id_to_deps)?;

        // Build result
        let mut result = Vec::new();
        for id in sorted_ids {
            if let Some(step) = id_to_step.remove(&id) {
                result.push(step);
            }
        }

        Ok(result)
    }

    /// Topological sort using Kahn's algorithm
    fn topological_sort(&self, deps: &HashMap<String, Vec<String>>) -> Result<Vec<String>, ContextError> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for id in deps.keys() {
            in_degree.entry(id.clone()).or_insert(0);
            graph.entry(id.clone()).or_insert_with(Vec::new);
        }

        // Build graph and count in-degrees
        for (id, dep_list) in deps {
            for dep in dep_list {
                if deps.contains_key(dep) {
                    graph.entry(dep.clone()).or_default().push(id.clone());
                    *in_degree.entry(id.clone()).or_insert(0) += 1;
                }
            }
        }

        // Find nodes with no dependencies
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();
        let mut visited = HashSet::new();

        while let Some(id) = queue.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id.clone());
            result.push(id.clone());

            if let Some(dependents) = graph.get(&id) {
                for dep in dependents {
                    if let Some(degree) = in_degree.get_mut(dep) {
                        *degree = degree.saturating_sub(1);
                        if *degree == 0 && !visited.contains(dep) {
                            queue.push(dep.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if self.detect_cycles && result.len() != deps.len() {
            return Err(ContextError::ParseError(
                "Circular dependency detected in steps".to_string()
            ));
        }

        Ok(result)
    }

    /// Find parallel groups (steps that can execute together)
    pub fn find_parallel_groups<'a>(&self, steps: &'a [Step]) -> Vec<Vec<&'a Step>> {
        let mut groups = Vec::new();
        let mut completed: HashSet<String> = HashSet::new();
        let remaining: Vec<_> = steps.iter().collect();

        while completed.len() < steps.len() {
            let mut group = Vec::new();

            for step in &remaining {
                let id = step.id.as_str();
                if completed.contains(id) {
                    continue;
                }

                let deps_met = step.depends_on.iter().all(|d| completed.contains(d.as_str()));
                if deps_met {
                    group.push(*step);
                }
            }

            if group.is_empty() {
                break;
            }

            for step in &group {
                completed.insert(step.id.as_str().to_string());
            }

            groups.push(group);
        }

        groups
    }
}

impl Default for StepOrderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PackageId, StepType};

    #[test]
    fn order_empty() {
        let orderer = StepOrderer::new();
        let result = orderer.order(vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn order_with_dependencies() {
        let pkg_id = PackageId::new("TEST-001");
        let step1 = Step::new(StepId::new(&pkg_id, 1), "Step 1", StepType::CreateFile);
        let step2 = Step::new(StepId::new(&pkg_id, 2), "Step 2", StepType::ModifyFile)
            .depends_on(StepId::new(&pkg_id, 1));

        let orderer = StepOrderer::new();
        let result = orderer.order(vec![step2, step1]).unwrap();

        // Step 1 should come before step 2
        assert_eq!(result[0].id.as_str(), "TEST-001-step-001");
        assert_eq!(result[1].id.as_str(), "TEST-001-step-002");
    }

    #[test]
    fn find_parallel() {
        let pkg_id = PackageId::new("TEST-001");
        let step1 = Step::new(StepId::new(&pkg_id, 1), "Step 1", StepType::CreateFile);
        let step2 = Step::new(StepId::new(&pkg_id, 2), "Step 2", StepType::CreateFile);
        let step3 = Step::new(StepId::new(&pkg_id, 3), "Step 3", StepType::ModifyFile)
            .depends_on(StepId::new(&pkg_id, 1))
            .depends_on(StepId::new(&pkg_id, 2));

        let orderer = StepOrderer::new();
        let steps = [step1, step2, step3];
        let groups = orderer.find_parallel_groups(&steps);

        // First group should have step1 and step2 (parallel)
        // Second group should have step3 (after both complete)
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 2);
        assert_eq!(groups[1].len(), 1);
    }
}
