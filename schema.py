from dataclasses import dataclass
from typing import Optional


@dataclass
class Column:
    name: str
    column_type: str
    is_nullable: bool
    column_default: Optional[str]
    extra: Optional[str]
    column_key: Optional[str]

    def to_sql(self) -> str:
        parts = [f"`{self.name}`", self.column_type]
        parts.append("NULL" if self.is_nullable else "NOT NULL")
        if self.column_default is not None:
            upper = self.column_default.upper()
            if upper.startswith("CURRENT_TIMESTAMP") or upper == "NULL" or "'" not in self.column_default:
                parts.append(f"DEFAULT {self.column_default}")
            else:
                escaped = self.column_default.replace("'", "\\'")
                parts.append(f"DEFAULT '{escaped}'")
        if self.extra:
            parts.append(self.extra.upper())
        return " ".join(parts)


@dataclass
class Table:
    name: str
    columns: list[Column]
    create_sql: Optional[str] = None
