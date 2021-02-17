type User struct {
	name string
}

func (User u) Role() []string {
	rows := db.QueryContext(ctx, "SELECT role FROM user_roles WHERE username=?", u.name)
	names := make([]string, 0)
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			log.Fatal(err)
		}
		names = append(names, name)
	}
	return names
}