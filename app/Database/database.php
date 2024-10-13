<?php

namespace App\Database;

use PDO;
use Exception;

class Database
{
    private static $instance = null;
    private $pdo;

    private function __construct()
    {
        switch ($_ENV['DB_CONNECTOR']) {
            case 'pgsql':
                $this->pdo = new PDO(
                    'pgsql:host=' . $_ENV['DB_HOST'] .
                        ';port=' . $_ENV['DB_PORT'] .
                        ';dbname=' . $_ENV['DB_DATABASE'] .
                        ';sslmode=' . $_ENV['DB_SSL_MODE'],
                    $_ENV['DB_USERNAME'],
                    $_ENV['DB_PASSWORD']
                );

                break;

            case 'mysql':
                $dsn = 'mysql:host=' . $_ENV['DB_HOST'] .
                    ';port=' . $_ENV['DB_PORT'] .
                    ';dbname=' . $_ENV['DB_DATABASE'];

                $this->pdo = new PDO($dsn, $_ENV['DB_USERNAME'], $_ENV['DB_PASSWORD']);

                /*
                if (!empty($_ENV['DB_SSL_CA']) && !empty($_ENV['DB_SSL_CERT']) && !empty($_ENV['DB_SSL_KEY'])) {
                    $this->pdo->setAttribute(PDO::MYSQL_ATTR_SSL_CA, $_ENV['DB_SSL_CA']);
                    $this->pdo->setAttribute(PDO::MYSQL_ATTR_SSL_CERT, $_ENV['DB_SSL_CERT']);
                    $this->pdo->setAttribute(PDO::MYSQL_ATTR_SSL_KEY, $_ENV['DB_SSL_KEY']);
                }
                */

                /*
                if ($_ENV['DB_SSL_MODE'] === 'required') {
                    $dsn .= '?ssl-mode=required';
                }
                */
                break;

            default:
                throw new Exception("Unknown database connector");
        }

        $this->pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Retrieves the singleton instance of the PDO database connection.
     *
     * This method creates a new instance of the Database class if it doesn't exist yet.
     * It then returns the PDO instance stored in the Database object.
     *
     * @return PDO The PDO instance representing the database connection.
     *
     * @throws Exception If the database connector is unknown.
     */
    public static function getInstance(): PDO
    {
        if (self::$instance === null) {
            self::$instance = new Database();
        }
        return self::$instance->pdo;
    }

    /**
     * Executes a raw SQL query and returns the result as an array of objects.
     *
     * @param string $sql_string The SQL query to be executed.
     *
     * @return array An array of objects representing the result set.
     *
     * @throws PDOException If there is an error executing the SQL query.
     */
    public static function rawQuery(string $sql_string): array
    {
        $stmt = self::getInstance()->query($sql_string);

        return $stmt->fetchAll(PDO::FETCH_OBJ);
    }

    /**
     * Executes a prepared SQL statement with parameters and returns the result as an array of objects.
     *
     * @param string $sql_string The SQL statement to be executed.
     * @param array $parameters An optional array of parameters to bind to the SQL statement.
     *
     * @return array An array of objects representing the result set.
     *
     * @throws PDOException If there is an error executing the SQL statement.
     */
    public static function preparedQuery(string $sql_string, array $parameters = []): array
    {
        $stmt = self::getInstance()->prepare($sql_string);
        $stmt->execute($parameters);

        return $stmt->fetchAll(PDO::FETCH_OBJ);
    }

    /**
     * Tests the database connection by executing a simple query.
     *
     * @return bool Returns true if the connection is successful, false otherwise.
     *
     * @throws Exception If there is an error executing the SQL query.
     */
    public static function test(): bool
    {
        try {
            $pdo = self::getInstance();
            $pdo->query('SELECT 1');
            return true;
        } catch (Exception $e) {
            return false;
        }
    }
}
