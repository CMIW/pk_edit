pub struct Gen3Bag;

impl Gen3Bag {
    /// Returns regular bag items (excludes Key Items, Pokéballs, Berries, and TMs/HMs).
    fn items() -> Result<Vec<String>> {
        let conn = Connection::open("pk_edit.db")?;

        let mut stmt = conn.prepare("SELECT e_name FROM Items WHERE id_g3 IS NOT NULL AND type != 'Key Items' AND type != 'Pokeballs' AND type != 'Berries' AND type != 'Machines'")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut res = Vec::new();
        for result in rows {
            res.push(result?);
        }
        res.push(String::from("Nothing"));

        stmt.finalize()?;

        let _ = conn.close();

        Ok(res)
    }

    /// Returns all Pokéball names available in Gen III.
    fn balls(&self) -> Result<Vec<String>> {
        let conn = Connection::open("pk_edit.db")?;

        let mut stmt = conn
            .prepare("SELECT e_name FROM Items WHERE id_g3 IS NOT NULL AND type == 'Pokeballs'")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut res = Vec::new();
        for result in rows {
            res.push(result?);
        }
        res.push(String::from("Nothing"));

        stmt.finalize()?;

        let _ = conn.close();

        Ok(res)
    }

    /// Returns all Berry names available in Gen III.
    fn berries(&self) -> Result<Vec<String>> {
        let conn = Connection::open("pk_edit.db")?;

        let mut stmt =
            conn.prepare("SELECT e_name FROM Items WHERE id_g3 IS NOT NULL AND type == 'Berries'")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut res = Vec::new();
        for result in rows {
            res.push(result?);
        }
        res.push(String::from("Nothing"));

        stmt.finalize()?;

        let _ = conn.close();

        Ok(res)
    }

    /// Returns all TM and HM names available in Gen III.
    fn tms(&self) -> Result<Vec<String>> {
        let conn = Connection::open("pk_edit.db")?;

        let mut stmt = conn
            .prepare("SELECT e_name FROM Items WHERE id_g3 IS NOT NULL AND type == 'Machines'")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut res = Vec::new();
        for result in rows {
            res.push(result?);
        }
        res.push(String::from("Nothing"));

        stmt.finalize()?;

        let _ = conn.close();

        Ok(res)
    }

    /// Returns all Key Item names available in Gen III.
    fn key_items(&self) -> Result<Vec<String>> {
        let conn = Connection::open("pk_edit.db")?;

        let mut stmt = conn
            .prepare("SELECT e_name FROM Items WHERE id_g3 IS NOT NULL AND type == 'Key Items'")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut res = Vec::new();
        for result in rows {
            res.push(result?);
        }
        res.push(String::from("Nothing"));

        stmt.finalize()?;

        let _ = conn.close();

        Ok(res)
    }
}
