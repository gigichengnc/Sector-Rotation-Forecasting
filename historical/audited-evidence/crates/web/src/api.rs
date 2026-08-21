use crate::{WebError, WebResult};
use axum::{
    extract::{Path, Query, State, Multipart},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use rrg_calc::{RRGCalculator, AlertSystem, AlertConfig, Alert, RRGData, PortfolioAnalysis};
use rrg_data::{ETFData, DataFetcher, DataCache, Portfolio, PortfolioImporter, AssetManager, AssetInfo};
// use rrg_ml::{LSTMPredictor, ScenarioEngine, ScenarioConfig};  // Temporarily disabled
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub rrg_calculator: Arc<RRGCalculator>,
    pub data_fetcher: Arc<RwLock<DataFetcher>>,
    pub data_cache: Arc<DataCache>,
    pub alert_system: Arc<RwLock<AlertSystem>>,
    // ML components temporarily disabled due to dependency conflicts
    // pub lstm_predictor: Arc<RwLock<LSTMPredictor>>,
    // pub scenario_engine: Arc<ScenarioEngine>,
    pub rrg_data_cache: Arc<RwLock<HashMap<String, RRGData>>>,
    pub asset_manager: Arc<RwLock<AssetManager>>,
    pub portfolios: Arc<RwLock<HashMap<String, Portfolio>>>,
    pub websocket_manager: Arc<crate::WebSocketManager>,
}

impl AppState {
    pub fn new(cache_path: &std::path::Path, websocket_manager: Arc<crate::WebSocketManager>) -> WebResult<Self> {
        Ok(Self {
            rrg_calculator: Arc::new(RRGCalculator::new()),
            data_fetcher: Arc::new(RwLock::new(DataFetcher::new())),
            data_cache: Arc::new(DataCache::new(cache_path)?),
            alert_system: Arc::new(RwLock::new(AlertSystem::with_default_config())),
            // ML components temporarily disabled due to dependency conflicts
            // lstm_predictor: Arc::new(RwLock::new(LSTMPredictor::new(10, 50, 2))),
            // scenario_engine: Arc::new(ScenarioEngine::new()),
            rrg_data_cache: Arc::new(RwLock::new(HashMap::new())),
            asset_manager: Arc::new(RwLock::new(AssetManager::new())),
            portfolios: Arc::new(RwLock::new(HashMap::new())),
            websocket_manager,
        })
    }
}

/// Create API router with all endpoints
pub fn create_api_router() -> Router<AppState> {
    Router::new()
        // Basic health endpoints
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/status", get(get_system_status))
        
        // Data endpoints (working)
        .route("/api/v1/data/etfs", get(list_etfs))
        .route("/api/v1/data/etf/:symbol", get(get_etf_data))
        .route("/api/v1/data/etf/:symbol/refresh", post(refresh_etf_data))
        
        // Asset management endpoints (working)
        .route("/api/v1/assets/validate/:symbol", get(validate_asset))
        .route("/api/v1/assets/search", get(search_assets))
        .route("/api/v1/assets/benchmarks", get(get_benchmarks))
        
        // Portfolio management endpoints (working)
        .route("/api/v1/portfolios", get(list_portfolios))
        .route("/api/v1/portfolios", post(create_portfolio))
        .route("/api/v1/portfolios/:name", get(get_portfolio))
        .route("/api/v1/portfolios/:name", post(update_portfolio))
        .route("/api/v1/portfolios/:name", delete(delete_portfolio))
        
        // Temporarily disabled endpoints that have issues
        // .route("/api/v1/portfolios/:name/import", post(import_portfolio_csv))
        // .route("/api/v1/portfolios/:name/export", get(export_portfolio_csv))
        // .route("/api/v1/portfolios/:name/rrg", post(calculate_portfolio_rrg))
        // .route("/api/v1/portfolios/:name/analysis", get(get_portfolio_analysis))
        
        // RRG calculation endpoints (working)
        .route("/api/v1/rrg/calculate", post(calculate_rrg))
        .route("/api/v1/rrg/data/:symbol", get(get_rrg_data))
        .route("/api/v1/rrg/batch", post(calculate_batch_rrg))
        .route("/api/v1/rrg/sectors", get(get_sector_analysis))
        
        // Alert endpoints (working)
        .route("/api/v1/alerts", get(get_alerts))
        .route("/api/v1/alerts/config", get(get_alert_config))
        .route("/api/v1/alerts/config", post(update_alert_config))
        .route("/api/v1/alerts/history", get(get_alert_history))
        
        // ML prediction endpoints (placeholder implementations)
        .route("/api/v1/ml/predict/:symbol", post(predict_rrg))
        .route("/api/v1/ml/model/status", get(get_model_status))
        .route("/api/v1/ml/model/retrain", post(retrain_model))
        
        // Scenario simulation endpoints (placeholder implementations)
        .route("/api/v1/scenarios/simulate", post(simulate_scenario))
        .route("/api/v1/scenarios/compare", post(compare_scenarios))
        
        // Visualization endpoints (placeholder implementations)
        .route("/api/v1/viz/chart/:symbol", get(get_chart_data))
        .route("/api/v1/viz/interactive", post(get_interactive_data))
        
        // System endpoints (working)
        .route("/api/v1/system/metrics", get(get_system_metrics))
}

/// Basic health check handler
async fn health_check() -> &'static str {
    "OK"
}

// Request/Response types

#[derive(Deserialize)]
pub struct RRGCalculationRequest {
    pub etf_symbol: String,
    pub benchmark_symbol: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct RRGCalculationResponse {
    pub symbol: String,
    pub sector: String,
    pub data_points: usize,
    pub current_quadrant: String,
    pub quadrant_strength: f64,
    pub rs_ratio: Vec<f64>,
    pub rs_momentum: Vec<f64>,
    pub normalized_rs_ratio: Vec<f64>,
    pub normalized_rs_momentum: Vec<f64>,
    pub timestamps: Vec<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct CreatePortfolioRequest {
    pub name: String,
    pub benchmark_symbol: String,
}

#[derive(Serialize)]
pub struct PortfolioResponse {
    pub name: String,
    pub holdings: HashMap<String, HoldingResponse>,
    pub benchmark_symbol: String,
    pub total_holdings: usize,
    pub total_value: f64,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct HoldingResponse {
    pub symbol: String,
    pub quantity: f64,
    pub cost_basis: f64,
    pub current_weight: f64,
    pub asset_type: String,
    pub market_value: f64,
}

#[derive(Deserialize)]
pub struct AddHoldingRequest {
    pub symbol: String,
    pub quantity: f64,
    pub cost_basis: f64,
    pub asset_type: Option<String>,
}

#[derive(Serialize)]
pub struct PortfolioRRGResponse {
    pub portfolio_name: String,
    pub portfolio_rrg: RRGCalculationResponse,
    pub individual_assets: Vec<RRGCalculationResponse>,
    pub asset_contributions: Vec<AssetContributionResponse>,
    pub benchmark_symbol: String,
    pub calculation_time_ms: u64,
}

#[derive(Serialize)]
pub struct AssetContributionResponse {
    pub symbol: String,
    pub weight: f64,
    pub rs_ratio_contribution: f64,
    pub rs_momentum_contribution: f64,
    pub quadrant: String,
    pub performance_impact: f64,
}

#[derive(Serialize)]
pub struct AssetValidationResponse {
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub market_cap: Option<f64>,
    pub exchange: Option<String>,
    pub currency: Option<String>,
    pub is_valid: bool,
}

#[derive(Deserialize)]
pub struct AssetSearchQuery {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct BenchmarkInfo {
    pub symbol: String,
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct BatchRRGRequest {
    pub etf_symbols: Vec<String>,
    pub benchmark_symbol: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct BatchRRGResponse {
    pub results: Vec<RRGCalculationResponse>,
    pub errors: Vec<String>,
    pub benchmark: String,
    pub calculation_time_ms: u64,
}

#[derive(Deserialize)]
pub struct PredictionRequest {
    pub symbol: String,
    pub forecast_days: usize,
    pub confidence_interval: Option<f64>,
}

#[derive(Serialize)]
pub struct PredictionResponse {
    pub symbol: String,
    pub forecast_days: usize,
    pub predictions: Vec<RRGPrediction>,
    pub confidence_interval: f64,
    pub model_accuracy: f64,
}

#[derive(Serialize)]
pub struct RRGPrediction {
    pub timestamp: DateTime<Utc>,
    pub predicted_rs_ratio: f64,
    pub predicted_rs_momentum: f64,
    pub predicted_quadrant: String,
    pub confidence: f64,
}

#[derive(Deserialize)]
pub struct ScenarioRequest {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, f64>,
    pub symbols: Vec<String>,
    pub duration_days: usize,
}

#[derive(Serialize)]
pub struct ScenarioResponse {
    pub scenario_id: Uuid,
    pub name: String,
    pub results: Vec<ScenarioResult>,
    pub summary: ScenarioSummary,
}

#[derive(Serialize)]
pub struct ScenarioResult {
    pub symbol: String,
    pub baseline_performance: f64,
    pub scenario_performance: f64,
    pub performance_delta: f64,
    pub risk_metrics: RiskMetrics,
}

#[derive(Serialize)]
pub struct ScenarioSummary {
    pub total_symbols: usize,
    pub avg_performance_delta: f64,
    pub best_performer: String,
    pub worst_performer: String,
    pub scenario_probability: f64,
}

#[derive(Serialize)]
pub struct RiskMetrics {
    pub volatility: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
}

#[derive(Serialize)]
pub struct SystemStatus {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub data_cache_size: usize,
    pub active_alerts: usize,
    pub model_status: String,
    pub last_data_update: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct SystemMetrics {
    pub requests_per_minute: f64,
    pub avg_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
    pub memory_usage_mb: f64,
}

#[derive(Deserialize)]
pub struct QueryParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

// API Handlers

/// List available ETFs
pub async fn list_etfs(State(state): State<AppState>) -> WebResult<Json<Vec<String>>> {
    let etfs = vec![
        "XLK".to_string(), // Technology
        "XLF".to_string(), // Financial
        "XLE".to_string(), // Energy
        "XLV".to_string(), // Health Care
        "XLI".to_string(), // Industrial
        "XLY".to_string(), // Consumer Discretionary
        "XLP".to_string(), // Consumer Staples
        "XLB".to_string(), // Materials
        "XLU".to_string(), // Utilities
        "XLRE".to_string(), // Real Estate
        "XLC".to_string(), // Communication Services
    ];
    
    Ok(Json(etfs))
}

/// Get ETF data
pub async fn get_etf_data(
    Path(symbol): Path<String>,
    State(state): State<AppState>,
) -> WebResult<Json<ETFData>> {
    // Try cache first
    if state.data_cache.is_cached(&symbol) {
        let data = state.data_cache.retrieve_data(&symbol)
            .map_err(|e| WebError::server_error(format!("Cache retrieval failed: {}", e)))?;
        return Ok(Json(data));
    }
    
    // Fetch from API
    let mut data_fetcher = state.data_fetcher.write().await;
    let data = data_fetcher.fetch_asset_data(&symbol, "1y").await
        .map_err(|e| WebError::server_error(format!("Data fetch failed: {}", e)))?;
    
    // Cache the result
    if let Err(e) = state.data_cache.store_data(&data) {
        tracing::warn!("Failed to cache data for {}: {}", symbol, e);
    }
    
    Ok(Json(data))
}

/// Refresh ETF data (force fetch from API)
pub async fn refresh_etf_data(
    Path(symbol): Path<String>,
    State(state): State<AppState>,
) -> WebResult<Json<ETFData>> {
    let mut data_fetcher = state.data_fetcher.write().await;
    let data = data_fetcher.fetch_asset_data(&symbol, "1y").await
        .map_err(|e| WebError::server_error(format!("Data fetch failed: {}", e)))?;
    
    // Update cache
    if let Err(e) = state.data_cache.store_data(&data) {
        tracing::warn!("Failed to cache refreshed data for {}: {}", symbol, e);
    }
    
    Ok(Json(data))
}

// Asset Management Endpoints

/// Validate a single asset symbol
pub async fn validate_asset(
    Path(symbol): Path<String>,
    State(state): State<AppState>,
) -> WebResult<Json<AssetValidationResponse>> {
    let mut asset_manager = state.asset_manager.write().await;
    let asset_info = asset_manager.validate_symbol(&symbol).await
        .map_err(|e| WebError::server_error(format!("Asset validation failed: {}", e)))?;
    
    let response = AssetValidationResponse {
        symbol: asset_info.symbol,
        name: asset_info.name,
        asset_type: format!("{:?}", asset_info.asset_type),
        sector: asset_info.sector,
        market_cap: asset_info.market_cap,
        exchange: asset_info.exchange,
        currency: asset_info.currency,
        is_valid: asset_info.is_valid,
    };
    
    Ok(Json(response))
}

/// Search for assets by query
pub async fn search_assets(
    Query(query): Query<AssetSearchQuery>,
    State(state): State<AppState>,
) -> WebResult<Json<Vec<AssetValidationResponse>>> {
    let asset_manager = state.asset_manager.read().await;
    let results = asset_manager.search_symbols(&query.query);
    
    let limit = query.limit.unwrap_or(10).min(50); // Cap at 50 results
    let responses: Vec<AssetValidationResponse> = results
        .into_iter()
        .take(limit)
        .map(|asset_info| AssetValidationResponse {
            symbol: asset_info.symbol,
            name: asset_info.name,
            asset_type: format!("{:?}", asset_info.asset_type),
            sector: asset_info.sector,
            market_cap: asset_info.market_cap,
            exchange: asset_info.exchange,
            currency: asset_info.currency,
            is_valid: asset_info.is_valid,
        })
        .collect();
    
    Ok(Json(responses))
}

/// Get available benchmark options
pub async fn get_benchmarks(
    State(state): State<AppState>,
) -> WebResult<Json<Vec<BenchmarkInfo>>> {
    let benchmarks = vec![
        BenchmarkInfo {
            symbol: "SPY".to_string(),
            name: "SPDR S&P 500 ETF".to_string(),
            description: "Tracks the S&P 500 index".to_string(),
        },
        BenchmarkInfo {
            symbol: "QQQ".to_string(),
            name: "Invesco QQQ ETF".to_string(),
            description: "Tracks the NASDAQ-100 index".to_string(),
        },
        BenchmarkInfo {
            symbol: "VTI".to_string(),
            name: "Vanguard Total Stock Market ETF".to_string(),
            description: "Tracks the entire U.S. stock market".to_string(),
        },
        BenchmarkInfo {
            symbol: "DIA".to_string(),
            name: "SPDR Dow Jones Industrial Average ETF".to_string(),
            description: "Tracks the Dow Jones Industrial Average".to_string(),
        },
        BenchmarkInfo {
            symbol: "IWM".to_string(),
            name: "iShares Russell 2000 ETF".to_string(),
            description: "Tracks the Russell 2000 small-cap index".to_string(),
        },
    ];
    
    Ok(Json(benchmarks))
}

// Portfolio Management Endpoints

/// List all portfolios
pub async fn list_portfolios(
    State(state): State<AppState>,
) -> WebResult<Json<Vec<String>>> {
    let portfolios = state.portfolios.read().await;
    let portfolio_names: Vec<String> = portfolios.keys().cloned().collect();
    Ok(Json(portfolio_names))
}

/// Create a new portfolio
pub async fn create_portfolio(
    State(state): State<AppState>,
    Json(request): Json<CreatePortfolioRequest>,
) -> WebResult<Json<PortfolioResponse>> {
    let mut portfolios = state.portfolios.write().await;
    
    if portfolios.contains_key(&request.name) {
        return Err(WebError::bad_request(format!("Portfolio '{}' already exists", request.name)));
    }
    
    let portfolio = Portfolio::new(request.name.clone(), request.benchmark_symbol);
    let response = portfolio_to_response(&portfolio);
    
    portfolios.insert(request.name, portfolio);
    
    Ok(Json(response))
}

/// Get portfolio details
pub async fn get_portfolio(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> WebResult<Json<PortfolioResponse>> {
    let portfolios = state.portfolios.read().await;
    
    if let Some(portfolio) = portfolios.get(&name) {
        Ok(Json(portfolio_to_response(portfolio)))
    } else {
        Err(WebError::not_found(format!("Portfolio: {}", name)))
    }
}

/// Update portfolio (add/remove holdings)
pub async fn update_portfolio(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<AddHoldingRequest>,
) -> WebResult<Json<PortfolioResponse>> {
    let mut portfolios = state.portfolios.write().await;
    
    if let Some(portfolio) = portfolios.get_mut(&name) {
        // Validate the asset first
        let mut asset_manager = state.asset_manager.write().await;
        let asset_info = asset_manager.validate_symbol(&request.symbol).await
            .map_err(|e| WebError::server_error(format!("Asset validation failed: {}", e)))?;
        
        if !asset_info.is_valid {
            return Err(WebError::bad_request(format!("Invalid asset symbol: {}", request.symbol)));
        }
        
        // Determine asset type
        let asset_type = match request.asset_type.as_deref() {
            Some("Stock") => rrg_data::portfolio::AssetType::Stock,
            Some("ETF") => rrg_data::portfolio::AssetType::ETF,
            Some("Index") => rrg_data::portfolio::AssetType::Index,
            Some(other) => rrg_data::portfolio::AssetType::Other(other.to_string()),
            None => asset_info.asset_type,
        };
        
        let holding = rrg_data::portfolio::Holding {
            symbol: request.symbol,
            quantity: request.quantity,
            cost_basis: request.cost_basis,
            current_weight: 0.0, // Will be recalculated
            asset_type,
        };
        
        portfolio.add_holding(holding)
            .map_err(|e| WebError::server_error(format!("Failed to add holding: {}", e)))?;
        
        Ok(Json(portfolio_to_response(portfolio)))
    } else {
        Err(WebError::not_found(format!("Portfolio: {}", name)))
    }
}

/// Delete a portfolio
pub async fn delete_portfolio(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> WebResult<StatusCode> {
    let mut portfolios = state.portfolios.write().await;
    
    if portfolios.remove(&name).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(WebError::not_found(format!("Portfolio: {}", name)))
    }
}

/// Import portfolio from CSV
pub async fn import_portfolio_csv(
    Path(name): Path<String>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> WebResult<Json<PortfolioResponse>> {
    // Extract CSV data from multipart form
    let mut csv_content = String::new();
    let mut benchmark_symbol = "SPY".to_string();
    
    while let Some(field) = multipart.next_field().await
        .map_err(|e| WebError::bad_request(format!("Invalid multipart data: {}", e)))? {
        
        let field_name = field.name().unwrap_or("").to_string();
        
        match field_name.as_str() {
            "csv_file" => {
                let data = field.bytes().await
                    .map_err(|e| WebError::bad_request(format!("Failed to read CSV file: {}", e)))?;
                csv_content = String::from_utf8(data.to_vec())
                    .map_err(|e| WebError::bad_request(format!("Invalid UTF-8 in CSV file: {}", e)))?;
            },
            "benchmark" => {
                benchmark_symbol = field.text().await
                    .map_err(|e| WebError::bad_request(format!("Failed to read benchmark: {}", e)))?;
            },
            _ => {} // Ignore unknown fields
        }
    }
    
    if csv_content.is_empty() {
        return Err(WebError::bad_request("No CSV file provided".to_string()));
    }
    
    // Parse CSV and create portfolio
    let portfolio = PortfolioImporter::from_csv_string(&csv_content, &name, &benchmark_symbol)
        .map_err(|e| WebError::server_error(format!("Failed to import portfolio: {}", e)))?;
    
    let response = portfolio_to_response(&portfolio);
    
    // Store portfolio
    let mut portfolios = state.portfolios.write().await;
    portfolios.insert(name, portfolio);
    
    Ok(Json(response))
}

/// Export portfolio to CSV
pub async fn export_portfolio_csv(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> WebResult<String> {
    let portfolios = state.portfolios.read().await;
    
    if let Some(portfolio) = portfolios.get(&name) {
        let csv_content = PortfolioImporter::to_csv_string(portfolio)
            .map_err(|e| WebError::server_error(format!("Failed to export portfolio: {}", e)))?;
        
        Ok(csv_content)
    } else {
        Err(WebError::not_found(format!("Portfolio: {}", name)))
    }
}

/// Calculate portfolio-weighted RRG
pub async fn calculate_portfolio_rrg(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<RRGCalculationRequest>,
) -> WebResult<Json<PortfolioRRGResponse>> {
    let start_time = std::time::Instant::now();
    
    let portfolios = state.portfolios.read().await;
    let portfolio = portfolios.get(&name)
        .ok_or_else(|| WebError::not_found(format!("Portfolio: {}", name)))?;
    
    // Fetch data for all portfolio assets
    let mut data_fetcher = state.data_fetcher.write().await;
    let portfolio_data = data_fetcher.fetch_portfolio_data(portfolio, "1y").await;
    
    // Fetch benchmark data
    let benchmark_data = data_fetcher.fetch_asset_data(&request.benchmark_symbol, "1y").await
        .map_err(|e| WebError::server_error(format!("Failed to fetch benchmark data: {}", e)))?;
    
    drop(data_fetcher); // Release the lock
    
    // Calculate individual RRG for each asset
    let mut individual_rrg_data = HashMap::new();
    let mut individual_responses = Vec::new();
    let mut errors = Vec::new();
    
    for (symbol, data_result) in portfolio_data {
        match data_result {
            Ok(asset_data) => {
                match state.rrg_calculator.calculate_rrg(&asset_data, &benchmark_data) {
                    Ok(rrg_data) => {
                        let response = RRGCalculationResponse {
                            symbol: rrg_data.symbol.clone(),
                            sector: rrg_data.sector.clone(),
                            data_points: rrg_data.rs_ratio.len(),
                            current_quadrant: format!("{:?}", rrg_data.current_quadrant),
                            quadrant_strength: rrg_data.quadrant_strength,
                            rs_ratio: rrg_data.rs_ratio.clone(),
                            rs_momentum: rrg_data.rs_momentum.clone(),
                            normalized_rs_ratio: rrg_data.normalized_rs_ratio.clone(),
                            normalized_rs_momentum: rrg_data.normalized_rs_momentum.clone(),
                            timestamps: rrg_data.timestamps.clone(),
                        };
                        individual_responses.push(response);
                        individual_rrg_data.insert(symbol, rrg_data);
                    },
                    Err(e) => errors.push(format!("{}: RRG calculation failed - {}", symbol, e)),
                }
            },
            Err(e) => errors.push(format!("{}: Data fetch failed - {}", symbol, e)),
        }
    }
    
    if individual_rrg_data.is_empty() {
        return Err(WebError::server_error("No valid RRG data calculated for portfolio assets".to_string()));
    }
    
    // Calculate portfolio-weighted RRG
    let portfolio_rrg_data = state.rrg_calculator.calculate_portfolio_rrg(
        portfolio,
        &individual_rrg_data,
        &request.benchmark_symbol,
    ).map_err(|e| WebError::server_error(format!("Portfolio RRG calculation failed: {}", e)))?;
    
    let portfolio_rrg_response = RRGCalculationResponse {
        symbol: portfolio_rrg_data.symbol.clone(),
        sector: portfolio_rrg_data.sector.clone(),
        data_points: portfolio_rrg_data.rs_ratio.len(),
        current_quadrant: format!("{:?}", portfolio_rrg_data.current_quadrant),
        quadrant_strength: portfolio_rrg_data.quadrant_strength,
        rs_ratio: portfolio_rrg_data.rs_ratio,
        rs_momentum: portfolio_rrg_data.rs_momentum,
        normalized_rs_ratio: portfolio_rrg_data.normalized_rs_ratio,
        normalized_rs_momentum: portfolio_rrg_data.normalized_rs_momentum,
        timestamps: portfolio_rrg_data.timestamps,
    };
    
    // Calculate asset contributions
    let asset_contributions: Vec<AssetContributionResponse> = portfolio.holdings
        .iter()
        .filter_map(|(symbol, holding)| {
            individual_rrg_data.get(symbol).map(|rrg_data| {
                AssetContributionResponse {
                    symbol: symbol.clone(),
                    weight: holding.current_weight,
                    rs_ratio_contribution: rrg_data.rs_ratio.last().unwrap_or(&100.0) * holding.current_weight,
                    rs_momentum_contribution: rrg_data.rs_momentum.last().unwrap_or(&100.0) * holding.current_weight,
                    quadrant: format!("{:?}", rrg_data.current_quadrant),
                    performance_impact: holding.current_weight * rrg_data.quadrant_strength,
                }
            })
        })
        .collect();
    
    let calculation_time = start_time.elapsed().as_millis() as u64;
    
    let response = PortfolioRRGResponse {
        portfolio_name: name,
        portfolio_rrg: portfolio_rrg_response,
        individual_assets: individual_responses,
        asset_contributions,
        benchmark_symbol: request.benchmark_symbol,
        calculation_time_ms: calculation_time,
    };
    
    Ok(Json(response))
}

/// Get portfolio analysis
pub async fn get_portfolio_analysis(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> WebResult<Json<serde_json::Value>> {
    let portfolios = state.portfolios.read().await;
    
    if let Some(portfolio) = portfolios.get(&name) {
        // This would use the portfolio analysis from the calculator
        // For now, return basic portfolio info
        let analysis = serde_json::json!({
            "portfolio_name": name,
            "total_holdings": portfolio.total_holdings(),
            "benchmark": portfolio.benchmark_symbol,
            "last_updated": portfolio.last_updated,
            "holdings_summary": portfolio.holdings.iter().map(|(symbol, holding)| {
                serde_json::json!({
                    "symbol": symbol,
                    "weight": holding.current_weight,
                    "asset_type": format!("{:?}", holding.asset_type)
                })
            }).collect::<Vec<_>>()
        });
        
        Ok(Json(analysis))
    } else {
        Err(WebError::not_found(format!("Portfolio: {}", name)))
    }
}

// Helper function to convert Portfolio to PortfolioResponse
fn portfolio_to_response(portfolio: &Portfolio) -> PortfolioResponse {
    let holdings: HashMap<String, HoldingResponse> = portfolio.holdings
        .iter()
        .map(|(symbol, holding)| {
            let market_value = holding.quantity * holding.cost_basis;
            (symbol.clone(), HoldingResponse {
                symbol: holding.symbol.clone(),
                quantity: holding.quantity,
                cost_basis: holding.cost_basis,
                current_weight: holding.current_weight,
                asset_type: format!("{:?}", holding.asset_type),
                market_value,
            })
        })
        .collect();
    
    let total_value: f64 = holdings.values().map(|h| h.market_value).sum();
    
    PortfolioResponse {
        name: portfolio.name.clone(),
        holdings,
        benchmark_symbol: portfolio.benchmark_symbol.clone(),
        total_holdings: portfolio.total_holdings(),
        total_value,
        created_at: portfolio.created_at,
        last_updated: portfolio.last_updated,
    }
}

/// Calculate RRG for a single ETF
pub async fn calculate_rrg(
    State(state): State<AppState>,
    Json(request): Json<RRGCalculationRequest>,
) -> WebResult<Json<RRGCalculationResponse>> {
    let start_time = std::time::Instant::now();
    
    // Get ETF data
    let etf_data = if state.data_cache.is_cached(&request.etf_symbol) {
        state.data_cache.retrieve_data(&request.etf_symbol)
            .map_err(|e| WebError::server_error(format!("Failed to retrieve ETF data: {}", e)))?
    } else {
        let mut data_fetcher = state.data_fetcher.write().await;
        let data = data_fetcher.fetch_asset_data(&request.etf_symbol, "1y").await
            .map_err(|e| WebError::server_error(format!("Failed to fetch ETF data: {}", e)))?;
        
        // Cache for future use
        let _ = state.data_cache.store_data(&data);
        data
    };
    
    // Get benchmark data
    let benchmark_data = if state.data_cache.is_cached(&request.benchmark_symbol) {
        state.data_cache.retrieve_data(&request.benchmark_symbol)
            .map_err(|e| WebError::server_error(format!("Failed to retrieve benchmark data: {}", e)))?
    } else {
        let mut data_fetcher = state.data_fetcher.write().await;
        let data = data_fetcher.fetch_asset_data(&request.benchmark_symbol, "1y").await
            .map_err(|e| WebError::server_error(format!("Failed to fetch benchmark data: {}", e)))?;
        
        let _ = state.data_cache.store_data(&data);
        data
    };
    
    // Calculate RRG
    let rrg_data = state.rrg_calculator.calculate_rrg(&etf_data, &benchmark_data)
        .map_err(|e| WebError::server_error(format!("RRG calculation failed: {}", e)))?;
    
    // Cache RRG result
    {
        let mut cache = state.rrg_data_cache.write().await;
        cache.insert(request.etf_symbol.clone(), rrg_data.clone());
    }
    
    // Process alerts
    {
        let mut alert_system = state.alert_system.write().await;
        let mut temp_data = rrg_data.clone();
        temp_data.points = vec![rrg_data.get_latest_point().unwrap_or_else(|| {
            rrg_calc::RRGPoint::new(
                request.etf_symbol.clone(),
                chrono::Utc::now(),
                100.0,
                100.0,
            )
        })];
        
        if let Ok(alerts) = alert_system.process_data(&temp_data) {
            if !alerts.is_empty() {
                tracing::info!("Generated {} alerts for {}", alerts.len(), request.etf_symbol);
            }
        }
    }
    
    let response = RRGCalculationResponse {
        symbol: rrg_data.symbol.clone(),
        sector: rrg_data.sector.clone(),
        data_points: rrg_data.rs_ratio.len(),
        current_quadrant: format!("{:?}", rrg_data.current_quadrant),
        quadrant_strength: rrg_data.quadrant_strength,
        rs_ratio: rrg_data.rs_ratio,
        rs_momentum: rrg_data.rs_momentum,
        normalized_rs_ratio: rrg_data.normalized_rs_ratio,
        normalized_rs_momentum: rrg_data.normalized_rs_momentum,
        timestamps: rrg_data.timestamps,
    };
    
    tracing::info!("RRG calculation for {} completed in {:?}", 
        request.etf_symbol, start_time.elapsed());
    
    Ok(Json(response))
}

/// Get cached RRG data
pub async fn get_rrg_data(
    Path(symbol): Path<String>,
    State(state): State<AppState>,
) -> WebResult<Json<RRGCalculationResponse>> {
    let cache = state.rrg_data_cache.read().await;
    
    if let Some(rrg_data) = cache.get(&symbol) {
        let response = RRGCalculationResponse {
            symbol: rrg_data.symbol.clone(),
            sector: rrg_data.sector.clone(),
            data_points: rrg_data.rs_ratio.len(),
            current_quadrant: format!("{:?}", rrg_data.current_quadrant),
            quadrant_strength: rrg_data.quadrant_strength,
            rs_ratio: rrg_data.rs_ratio.clone(),
            rs_momentum: rrg_data.rs_momentum.clone(),
            normalized_rs_ratio: rrg_data.normalized_rs_ratio.clone(),
            normalized_rs_momentum: rrg_data.normalized_rs_momentum.clone(),
            timestamps: rrg_data.timestamps.clone(),
        };
        
        Ok(Json(response))
    } else {
        Err(WebError::not_found(format!("RRG data for symbol: {}", symbol)))
    }
}

/// Calculate RRG for multiple ETFs
pub async fn calculate_batch_rrg(
    State(state): State<AppState>,
    Json(request): Json<BatchRRGRequest>,
) -> WebResult<Json<BatchRRGResponse>> {
    let start_time = std::time::Instant::now();
    let mut results = Vec::new();
    let mut errors = Vec::new();
    
    // Get benchmark data once
    let benchmark_data = if state.data_cache.is_cached(&request.benchmark_symbol) {
        state.data_cache.retrieve_data(&request.benchmark_symbol)
            .map_err(|e| WebError::server_error(format!("Failed to retrieve benchmark data: {}", e)))?
    } else {
        let mut data_fetcher = state.data_fetcher.write().await;
        let data = data_fetcher.fetch_asset_data(&request.benchmark_symbol, "1y").await
            .map_err(|e| WebError::server_error(format!("Failed to fetch benchmark data: {}", e)))?;
        
        let _ = state.data_cache.store_data(&data);
        data
    };
    
    // Process each ETF
    for symbol in &request.etf_symbols {
        match process_single_etf(&state, symbol, &benchmark_data).await {
            Ok(response) => results.push(response),
            Err(e) => errors.push(format!("{}: {}", symbol, e)),
        }
    }
    
    let calculation_time = start_time.elapsed().as_millis() as u64;
    
    let response = BatchRRGResponse {
        results,
        errors,
        benchmark: request.benchmark_symbol,
        calculation_time_ms: calculation_time,
    };
    
    Ok(Json(response))
}

async fn process_single_etf(
    state: &AppState,
    symbol: &str,
    benchmark_data: &ETFData,
) -> WebResult<RRGCalculationResponse> {
    // Get ETF data
    let etf_data = if state.data_cache.is_cached(symbol) {
        state.data_cache.retrieve_data(symbol)
            .map_err(|e| WebError::server_error(format!("Failed to retrieve data: {}", e)))?
    } else {
        let mut data_fetcher = state.data_fetcher.write().await;
        let data = data_fetcher.fetch_asset_data(symbol, "1y").await
            .map_err(|e| WebError::server_error(format!("Failed to fetch data: {}", e)))?;
        
        let _ = state.data_cache.store_data(&data);
        data
    };
    
    // Calculate RRG
    let rrg_data = state.rrg_calculator.calculate_rrg(&etf_data, benchmark_data)
        .map_err(|e| WebError::server_error(format!("RRG calculation failed: {}", e)))?;
    
    // Cache result
    {
        let mut cache = state.rrg_data_cache.write().await;
        cache.insert(symbol.to_string(), rrg_data.clone());
    }
    
    Ok(RRGCalculationResponse {
        symbol: rrg_data.symbol.clone(),
        sector: rrg_data.sector.clone(),
        data_points: rrg_data.rs_ratio.len(),
        current_quadrant: format!("{:?}", rrg_data.current_quadrant),
        quadrant_strength: rrg_data.quadrant_strength,
        rs_ratio: rrg_data.rs_ratio,
        rs_momentum: rrg_data.rs_momentum,
        normalized_rs_ratio: rrg_data.normalized_rs_ratio,
        normalized_rs_momentum: rrg_data.normalized_rs_momentum,
        timestamps: rrg_data.timestamps,
    })
}

/// Get sector analysis
pub async fn get_sector_analysis(
    State(state): State<AppState>,
) -> WebResult<Json<HashMap<String, Vec<String>>>> {
    let sectors = HashMap::from([
        ("Technology".to_string(), vec!["XLK".to_string()]),
        ("Financial".to_string(), vec!["XLF".to_string()]),
        ("Energy".to_string(), vec!["XLE".to_string()]),
        ("Health Care".to_string(), vec!["XLV".to_string()]),
        ("Industrial".to_string(), vec!["XLI".to_string()]),
        ("Consumer Discretionary".to_string(), vec!["XLY".to_string()]),
        ("Consumer Staples".to_string(), vec!["XLP".to_string()]),
        ("Materials".to_string(), vec!["XLB".to_string()]),
        ("Utilities".to_string(), vec!["XLU".to_string()]),
        ("Real Estate".to_string(), vec!["XLRE".to_string()]),
        ("Communication Services".to_string(), vec!["XLC".to_string()]),
    ]);
    
    Ok(Json(sectors))
}

/// Get current alerts
pub async fn get_alerts(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> WebResult<Json<Vec<Alert>>> {
    let alert_system = state.alert_system.read().await;
    let mut alerts: Vec<Alert> = alert_system.get_alert_history().to_vec();
    
    // Apply date filtering
    if let Some(start_date) = params.start_date {
        alerts.retain(|alert| alert.timestamp >= start_date);
    }
    
    if let Some(end_date) = params.end_date {
        alerts.retain(|alert| alert.timestamp <= end_date);
    }
    
    // Apply pagination
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100);
    
    alerts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Most recent first
    
    let paginated_alerts: Vec<Alert> = alerts
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    
    Ok(Json(paginated_alerts))
}

/// Get alert configuration
pub async fn get_alert_config(
    State(state): State<AppState>,
) -> WebResult<Json<AlertConfig>> {
    let alert_system = state.alert_system.read().await;
    // Note: This would need to be implemented in AlertSystem
    let config = AlertConfig::default(); // Placeholder
    Ok(Json(config))
}

/// Update alert configuration
pub async fn update_alert_config(
    State(state): State<AppState>,
    Json(config): Json<AlertConfig>,
) -> WebResult<Json<AlertConfig>> {
    let mut alert_system = state.alert_system.write().await;
    alert_system.update_config(config.clone());
    Ok(Json(config))
}

/// Get alert history
pub async fn get_alert_history(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> WebResult<Json<Vec<Alert>>> {
    // This is similar to get_alerts but might have different filtering logic
    get_alerts(State(state), Query(params)).await
}

/// Get system status
pub async fn get_system_status(
    State(state): State<AppState>,
) -> WebResult<Json<SystemStatus>> {
    let cache_size = {
        let cache = state.rrg_data_cache.read().await;
        cache.len()
    };
    
    let active_alerts = {
        let alert_system = state.alert_system.read().await;
        alert_system.get_recent_alerts(24).len()
    };
    
    let status = SystemStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // Would need to track actual uptime
        data_cache_size: cache_size,
        active_alerts,
        model_status: "ready".to_string(),
        last_data_update: Some(chrono::Utc::now()),
    };
    
    Ok(Json(status))
}

/// Get system metrics
pub async fn get_system_metrics(
    State(state): State<AppState>,
) -> WebResult<Json<SystemMetrics>> {
    // In a real implementation, these would be tracked by middleware
    let metrics = SystemMetrics {
        requests_per_minute: 0.0,
        avg_response_time_ms: 0.0,
        cache_hit_rate: 0.85,
        error_rate: 0.01,
        memory_usage_mb: 0.0,
    };
    
    Ok(Json(metrics))
}

// Placeholder implementations for ML endpoints
pub async fn predict_rrg(
    Path(symbol): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<PredictionRequest>,
) -> WebResult<Json<PredictionResponse>> {
    // Placeholder implementation
    let response = PredictionResponse {
        symbol: symbol.clone(),
        forecast_days: request.forecast_days,
        predictions: Vec::new(),
        confidence_interval: request.confidence_interval.unwrap_or(0.95),
        model_accuracy: 0.75,
    };
    
    Ok(Json(response))
}

pub async fn get_model_status(
    State(state): State<AppState>,
) -> WebResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "status": "ready",
        "last_training": "2024-01-01T00:00:00Z",
        "accuracy": 0.75
    })))
}

pub async fn retrain_model(
    State(state): State<AppState>,
) -> WebResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "status": "training_started",
        "estimated_completion": "2024-01-01T01:00:00Z"
    })))
}

pub async fn simulate_scenario(
    State(state): State<AppState>,
    Json(request): Json<ScenarioRequest>,
) -> WebResult<Json<ScenarioResponse>> {
    // Placeholder implementation
    let response = ScenarioResponse {
        scenario_id: Uuid::new_v4(),
        name: request.name,
        results: Vec::new(),
        summary: ScenarioSummary {
            total_symbols: request.symbols.len(),
            avg_performance_delta: 0.0,
            best_performer: "XLK".to_string(),
            worst_performer: "XLE".to_string(),
            scenario_probability: 0.65,
        },
    };
    
    Ok(Json(response))
}

pub async fn compare_scenarios(
    State(state): State<AppState>,
    Json(scenarios): Json<Vec<ScenarioRequest>>,
) -> WebResult<Json<Vec<ScenarioResponse>>> {
    // Placeholder implementation
    let responses = scenarios.into_iter().map(|req| ScenarioResponse {
        scenario_id: Uuid::new_v4(),
        name: req.name,
        results: Vec::new(),
        summary: ScenarioSummary {
            total_symbols: req.symbols.len(),
            avg_performance_delta: 0.0,
            best_performer: "XLK".to_string(),
            worst_performer: "XLE".to_string(),
            scenario_probability: 0.65,
        },
    }).collect();
    
    Ok(Json(responses))
}

pub async fn get_chart_data(
    Path(symbol): Path<String>,
    State(state): State<AppState>,
) -> WebResult<Json<serde_json::Value>> {
    // Placeholder implementation
    Ok(Json(serde_json::json!({
        "symbol": symbol,
        "chart_type": "rrg",
        "data": []
    })))
}

pub async fn get_interactive_data(
    State(state): State<AppState>,
    Json(symbols): Json<Vec<String>>,
) -> WebResult<Json<serde_json::Value>> {
    // Placeholder implementation
    Ok(Json(serde_json::json!({
        "symbols": symbols,
        "interactive_data": []
    })))
}