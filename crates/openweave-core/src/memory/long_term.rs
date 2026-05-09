use super::Memory;
use crate::error::Result;
use crate::llm::Message;
use std::cmp::Ordering;

pub struct LongTermMemory {
    // text, embedding, metadata
    records: Vec<(String, Vec<f32>, String)>,
}

impl LongTermMemory {
    /// Create a new, empty LongTermMemory.
    ///
    /// # Examples
    ///
    /// ```
    /// let mem = LongTermMemory::new();
    /// assert!(mem.search(&[], 1).is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    
    /// Searches stored records by cosine similarity against `query_embedding` and returns the highest-scoring texts.
    ///
    /// # Parameters
    ///
    /// - `query_embedding`: embedding vector used to score stored records against.
    /// - `limit`: maximum number of matching texts to return; if fewer records exist, returns all matches.
    ///
    /// # Returns
    ///
    /// A `Vec<String>` containing the texts of the top `limit` records ranked by cosine similarity (highest first). Metadata is not used in ranking or output.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut mem = LongTermMemory::new();
    /// mem.insert("apple".to_string(), vec![1.0, 0.0], "fruit".to_string());
    /// mem.insert("banana".to_string(), vec![0.9, 0.1], "fruit".to_string());
    /// let results = mem.search(&[1.0, 0.0], 2);
    /// assert!(results.contains(&"apple".to_string()));
    /// assert!(results.contains(&"banana".to_string()));
    /// ```
    pub fn search(&self, query_embedding: &[f32], limit: usize) -> Vec<String> {
        let mut scored_records: Vec<(&String, f32)> = self.records
            .iter()
            .map(|(text, emb, _meta)| {
                let score = cosine_similarity(query_embedding, emb);
                (text, score)
            })
            .collect();

        // Sort descending by score
        scored_records.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        scored_records
            .into_iter()
            .take(limit)
            .map(|(text, _score)| text.clone())
            .collect()
    }

    // Direct insertion method bypassing standard add() since we don't have embedder built-in
    /// Appends a long-term memory record containing text, its embedding, and associated metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut mem = LongTermMemory::new();
    /// mem.insert("hello".to_string(), vec![0.0_f32; 3], "greeting".to_string());
    /// // inserted record should be searchable (depending on embedding and query)
    /// assert_eq!(mem.records.len(), 1);
    /// ```
    pub fn insert(&mut self, text: String, embedding: Vec<f32>, metadata: String) {
        self.records.push((text, embedding, metadata));
    }
}

impl Default for LongTermMemory {
    /// Create a new `LongTermMemory` initialized with no records.
    ///
    /// # Examples
    ///
    /// ```
    /// let mem = LongTermMemory::default();
    /// assert!(mem.records.is_empty());
    /// ```
    ///
    /// # Returns
    ///
    /// `LongTermMemory` initialized with an empty record list.
    fn default() -> Self {
        Self::new()
    }
}

impl Memory for LongTermMemory {
    /// No-op `add` implementation that ignores the provided message.
    ///
    /// This implementation intentionally does not store the message or generate embeddings;
    /// it always succeeds without side effects.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut mem = LongTermMemory::new();
    /// let msg = Message::new("user", "hello");
    /// assert!(mem.add(msg).is_ok());
    /// ```
    fn add(&mut self, _message: Message) -> Result<()> {
        // Core trait assumes standard messages. To do this properly requires 
        // an async embedder injected into the memory, or passing embeddings.
        Ok(())
    }

    fn get_context(&self) -> Vec<Message> {
        // Context retrieval should be query-based for LTM
        Vec::new()
    }

    /// Removes all long-term memory records from this storage.
    ///
    /// This clears the internal records vector so subsequent searches or inserts
    /// operate on an empty store.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut mem = LongTermMemory::new();
    /// mem.insert("a".to_string(), vec![0.0; 3], "meta".to_string());
    /// mem.clear();
    /// assert!(mem.search(&[0.0, 0.0, 0.0], 10).is_empty());
    /// ```
    fn clear(&mut self) {
        self.records.clear();
    }
}

/// Computes the cosine similarity between two equal-length float slices.
///
/// Returns the cosine of the angle between the two vectors as a value in the range [-1.0, 1.0]. If the slices differ in length, are empty, or either vector has zero magnitude, this function returns `0.0`.
///
/// # Examples
///
/// ```
/// let a = [1.0_f32, 0.0];
/// let b = [0.0_f32, 1.0];
/// let orthogonal = super::cosine_similarity(&a, &b);
/// assert_eq!(orthogonal, 0.0);
///
/// let c = [1.0_f32, 1.0];
/// let d = [1.0_f32, 1.0];
/// let identical = super::cosine_similarity(&c, &d);
/// assert_eq!(identical, 1.0);
/// ```
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (x, y) in a.iter().zip(b.iter()) {
        dot_product += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}