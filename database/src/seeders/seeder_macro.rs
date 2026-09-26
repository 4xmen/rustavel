
#[macro_export]
macro_rules! seeders {
    () => {{
        title(TitleKind::Info, "Noting to seed");
        Ok::<(), anyhow::Error>(())
    }};

    ($($seeder:ty),+ $(,)?) => {{
        $(
            {
                // Start timing immediately before the seeder runs.
                // This measures only the execution time of this specific seeder.
                let start = Instant::now();

                // Run the seeder and wait for it to complete.
                let result = <$seeder>::run().await;

                match result {
                    Ok(_) => {
                        // If the seeder succeeded, print the same success-style message
                        // together with the elapsed time.
                        operation( stringify!($seeder),start.elapsed(), Status::Done );
                    }
                    Err(err) => {
                        // If the seeder failed, print the error plus the elapsed time.
                        // Then stop execution by returning the error to the caller.
                        operation( stringify!($seeder),start.elapsed(), Status::Failed );
                        return Err(err);
                    }
                }
            }
        )+

        Ok::<(), anyhow::Error>(())
    }};
}