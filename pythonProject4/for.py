while 1:
    name = input("name;")
    num = input("num")
    print("""dataset <- dataset %>%
      mutate_at(vars({}),
                list(~ ifelse(. %in% c({}), NA, .)))""".format(name, num))