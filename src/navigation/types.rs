use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationSpace {
    World,
    StoreLocal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavigationPoint {
    pub space: NavigationSpace,
    pub pos: Vec2,
}

#[derive(Debug, Clone)]
pub struct NavigationRoute {
    pub space: NavigationSpace,
    pub waypoints: VecDeque<Vec2>,
}

pub struct AgentNavigationProfile {
    pub radius: f32,
    pub clearance: f32,
    pub can_use_staff_only: bool,
}

pub struct RoutePlanRequest {
    pub start: NavigationPoint,
    pub target: NavigationPoint,
    pub agent: AgentNavigationProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutePlanError {
    StartOutsideWalkableArea,
    TargetOutsideWalkableArea,
    StartBlocked,
    TargetBlocked,
    NoRoute,
    UnsupportedSpace,
}

pub type RoutePlanResult = Result<NavigationRoute, RoutePlanError>;
