#!/usr/bin/env python3
"""
Knowledge Base Loader for Gold Loan Voice Agent

Loads knowledge documents from YAML files and indexes them in Qdrant vector store.
Optimized for small LLMs with multilingual embeddings.

Usage:
    python scripts/load_knowledge.py [--dry-run] [--validate-only]

Requirements:
    pip install qdrant-client sentence-transformers pyyaml rich
"""

import argparse
import hashlib
import json
import os
import sys
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Optional

import yaml
from qdrant_client import QdrantClient
from qdrant_client.http import models
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.table import Table
from sentence_transformers import SentenceTransformer

console = Console()

# Configuration
KNOWLEDGE_DIR = Path(__file__).parent.parent / "knowledge"
MANIFEST_FILE = KNOWLEDGE_DIR / "manifest.yaml"
QDRANT_HOST = os.getenv("QDRANT_HOST", "localhost")
QDRANT_PORT = int(os.getenv("QDRANT_PORT", "6333"))
COLLECTION_NAME = "gold_loan_knowledge"

# Embedding model - multilingual for Hindi/English
EMBEDDING_MODEL = "intfloat/multilingual-e5-small"
EMBEDDING_DIM = 384


@dataclass
class KnowledgeDocument:
    """A single knowledge document."""
    id: str
    title: str
    content: str
    category: str
    language: str
    keywords: list[str]
    segment_relevance: list[str]
    intent_triggers: list[str]
    file_source: str

    def to_payload(self) -> dict:
        """Convert to Qdrant payload format."""
        return {
            "id": self.id,
            "title": self.title,
            "content": self.content,
            "category": self.category,
            "language": self.language,
            "keywords": self.keywords,
            "segment_relevance": self.segment_relevance,
            "intent_triggers": self.intent_triggers,
            "file_source": self.file_source,
            "content_hash": hashlib.md5(self.content.encode()).hexdigest(),
            "indexed_at": datetime.utcnow().isoformat(),
        }


class KnowledgeLoader:
    """Loads and indexes knowledge documents."""

    def __init__(self, dry_run: bool = False):
        self.dry_run = dry_run
        self.documents: list[KnowledgeDocument] = []
        self.embedder: Optional[SentenceTransformer] = None
        self.qdrant: Optional[QdrantClient] = None

    def load_manifest(self) -> dict:
        """Load the knowledge manifest."""
        with open(MANIFEST_FILE) as f:
            return yaml.safe_load(f)

    def load_documents_from_file(self, file_path: Path) -> list[KnowledgeDocument]:
        """Load documents from a single YAML file."""
        docs = []

        try:
            with open(file_path) as f:
                data = yaml.safe_load(f)

            if not data or "documents" not in data:
                console.print(f"[yellow]Warning: No documents in {file_path.name}[/]")
                return docs

            for doc_data in data["documents"]:
                doc = KnowledgeDocument(
                    id=doc_data["id"],
                    title=doc_data["title"],
                    content=doc_data["content"].strip(),
                    category=doc_data.get("category", "general"),
                    language=doc_data.get("language", "en"),
                    keywords=doc_data.get("keywords", []),
                    segment_relevance=doc_data.get("segment_relevance", []),
                    intent_triggers=doc_data.get("intent_triggers", []),
                    file_source=file_path.name,
                )
                docs.append(doc)

        except Exception as e:
            console.print(f"[red]Error loading {file_path.name}: {e}[/]")

        return docs

    def load_all_documents(self) -> None:
        """Load all documents from knowledge files."""
        manifest = self.load_manifest()

        console.print("\n[bold]Loading knowledge files...[/]\n")

        for file_info in manifest["files"]:
            file_path = KNOWLEDGE_DIR / file_info["path"]

            if not file_path.exists():
                console.print(f"[yellow]Warning: File not found: {file_info['path']}[/]")
                continue

            docs = self.load_documents_from_file(file_path)
            self.documents.extend(docs)
            console.print(f"  [green]✓[/] {file_info['path']}: {len(docs)} documents")

        console.print(f"\n[bold]Total documents loaded: {len(self.documents)}[/]")

    def validate_documents(self) -> bool:
        """Validate all loaded documents."""
        console.print("\n[bold]Validating documents...[/]\n")

        errors = []
        warnings = []
        seen_ids = set()

        for doc in self.documents:
            # Check for duplicate IDs
            if doc.id in seen_ids:
                errors.append(f"Duplicate ID: {doc.id}")
            seen_ids.add(doc.id)

            # Check content length
            if len(doc.content) < 100:
                warnings.append(f"{doc.id}: Content too short ({len(doc.content)} chars)")
            if len(doc.content) > 1500:
                warnings.append(f"{doc.id}: Content too long ({len(doc.content)} chars)")

            # Check required fields
            if not doc.keywords:
                warnings.append(f"{doc.id}: No keywords defined")

        # Print results
        if errors:
            console.print("[red]Errors:[/]")
            for error in errors:
                console.print(f"  [red]✗[/] {error}")

        if warnings:
            console.print("\n[yellow]Warnings:[/]")
            for warning in warnings[:10]:  # Limit to first 10
                console.print(f"  [yellow]![/] {warning}")
            if len(warnings) > 10:
                console.print(f"  ... and {len(warnings) - 10} more warnings")

        if not errors and not warnings:
            console.print("[green]✓ All documents valid[/]")

        return len(errors) == 0

    def print_statistics(self) -> None:
        """Print document statistics."""
        console.print("\n[bold]Document Statistics[/]\n")

        # By category
        by_category = {}
        for doc in self.documents:
            by_category[doc.category] = by_category.get(doc.category, 0) + 1

        table = Table(title="By Category")
        table.add_column("Category", style="cyan")
        table.add_column("Count", style="green")
        for cat, count in sorted(by_category.items()):
            table.add_row(cat, str(count))
        console.print(table)

        # By language
        by_lang = {}
        for doc in self.documents:
            by_lang[doc.language] = by_lang.get(doc.language, 0) + 1

        console.print("\n[bold]By Language:[/]")
        for lang, count in sorted(by_lang.items()):
            console.print(f"  {lang}: {count} documents")

        # By segment
        segment_coverage = {}
        for doc in self.documents:
            for seg in doc.segment_relevance:
                segment_coverage[seg] = segment_coverage.get(seg, 0) + 1

        console.print("\n[bold]Segment Coverage:[/]")
        for seg, count in sorted(segment_coverage.items()):
            console.print(f"  {seg}: {count} documents")

    def initialize_embedder(self) -> None:
        """Initialize the embedding model."""
        if self.dry_run:
            return

        console.print(f"\n[bold]Loading embedding model: {EMBEDDING_MODEL}[/]")
        self.embedder = SentenceTransformer(EMBEDDING_MODEL)
        console.print("[green]✓ Model loaded[/]")

    def initialize_qdrant(self) -> None:
        """Initialize Qdrant client and collection."""
        if self.dry_run:
            return

        console.print(f"\n[bold]Connecting to Qdrant: {QDRANT_HOST}:{QDRANT_PORT}[/]")

        try:
            self.qdrant = QdrantClient(host=QDRANT_HOST, port=QDRANT_PORT)

            # Check if collection exists
            collections = self.qdrant.get_collections().collections
            exists = any(c.name == COLLECTION_NAME for c in collections)

            if exists:
                console.print(f"[yellow]Collection '{COLLECTION_NAME}' exists, will recreate[/]")
                self.qdrant.delete_collection(COLLECTION_NAME)

            # Create collection
            self.qdrant.create_collection(
                collection_name=COLLECTION_NAME,
                vectors_config=models.VectorParams(
                    size=EMBEDDING_DIM,
                    distance=models.Distance.COSINE,
                ),
            )

            # Create payload indexes for filtering
            self.qdrant.create_payload_index(
                collection_name=COLLECTION_NAME,
                field_name="category",
                field_schema=models.PayloadSchemaType.KEYWORD,
            )
            self.qdrant.create_payload_index(
                collection_name=COLLECTION_NAME,
                field_name="language",
                field_schema=models.PayloadSchemaType.KEYWORD,
            )

            console.print(f"[green]✓ Collection '{COLLECTION_NAME}' created[/]")

        except Exception as e:
            console.print(f"[red]Error connecting to Qdrant: {e}[/]")
            console.print("[yellow]Make sure Qdrant is running: docker run -p 6333:6333 qdrant/qdrant[/]")
            sys.exit(1)

    def index_documents(self) -> None:
        """Index all documents in Qdrant."""
        if self.dry_run:
            console.print("\n[yellow]Dry run - skipping indexing[/]")
            return

        console.print("\n[bold]Indexing documents...[/]")

        # Prepare texts for embedding
        texts = []
        for doc in self.documents:
            # Combine title and content for embedding
            # Use e5 format: "passage: {text}"
            text = f"passage: {doc.title}. {doc.content}"
            texts.append(text)

        # Generate embeddings with progress
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console,
        ) as progress:
            task = progress.add_task("Generating embeddings...", total=len(texts))

            # Batch embedding
            embeddings = self.embedder.encode(
                texts,
                batch_size=32,
                show_progress_bar=False,
            )
            progress.update(task, completed=len(texts))

        # Upload to Qdrant
        console.print("Uploading to Qdrant...")

        points = []
        for i, (doc, embedding) in enumerate(zip(self.documents, embeddings)):
            point = models.PointStruct(
                id=i,
                vector=embedding.tolist(),
                payload=doc.to_payload(),
            )
            points.append(point)

        # Batch upload
        batch_size = 100
        for i in range(0, len(points), batch_size):
            batch = points[i:i + batch_size]
            self.qdrant.upsert(
                collection_name=COLLECTION_NAME,
                points=batch,
            )

        console.print(f"[green]✓ Indexed {len(points)} documents[/]")

    def verify_index(self) -> None:
        """Verify the index with sample queries."""
        if self.dry_run or not self.qdrant:
            return

        console.print("\n[bold]Verifying index with sample queries...[/]")

        test_queries = [
            ("What is the gold loan interest rate?", "rate"),
            ("Is my gold safe at Kotak?", "safety"),
            ("How to switch from Muthoot?", "competitor"),
            ("Mahila ke liye gold loan", "segment"),
        ]

        for query, expected_category in test_queries:
            # Use e5 format for query
            query_text = f"query: {query}"
            embedding = self.embedder.encode([query_text])[0]

            results = self.qdrant.search(
                collection_name=COLLECTION_NAME,
                query_vector=embedding.tolist(),
                limit=3,
            )

            if results:
                top_result = results[0]
                match = "✓" if top_result.payload.get("category") == expected_category else "?"
                console.print(f"  [{('green' if match == '✓' else 'yellow')}]{match}[/] '{query}' → {top_result.payload.get('title', 'N/A')[:50]}")
            else:
                console.print(f"  [red]✗[/] '{query}' → No results")

    def run(self, validate_only: bool = False) -> None:
        """Run the knowledge loading process."""
        console.print("\n[bold blue]═══ Kotak Gold Loan Knowledge Base Loader ═══[/]\n")

        # Load documents
        self.load_all_documents()

        # Validate
        valid = self.validate_documents()
        if not valid:
            console.print("\n[red]Validation failed. Fix errors before indexing.[/]")
            sys.exit(1)

        # Print statistics
        self.print_statistics()

        if validate_only:
            console.print("\n[yellow]Validation only - skipping indexing[/]")
            return

        # Initialize and index
        self.initialize_embedder()
        self.initialize_qdrant()
        self.index_documents()
        self.verify_index()

        console.print("\n[bold green]✓ Knowledge base loaded successfully![/]\n")


def main():
    parser = argparse.ArgumentParser(description="Load knowledge base into Qdrant")
    parser.add_argument("--dry-run", action="store_true", help="Don't actually index")
    parser.add_argument("--validate-only", action="store_true", help="Only validate, don't index")
    args = parser.parse_args()

    loader = KnowledgeLoader(dry_run=args.dry_run)
    loader.run(validate_only=args.validate_only)


if __name__ == "__main__":
    main()
